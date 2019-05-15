#[macro_use]
extern crate lazy_static;
extern crate gdk;
extern crate gio;
extern crate gtk;
extern crate glib;
extern crate url;
extern crate compressor;
use compressor::{CompressionSpec, Scale, Quality, compress_specs, find_all_jpegs,getSpec};

use gio::prelude::*;
use gtk::prelude::*;
use url::Url;
use glib::Sender;

use gdk::DragAction;


use gtk::{Window, Builder, Button, MessageDialog, FileChooserDialog, FileChooserAction, ResponseType, TextView, DestDefaults, TargetFlags, MenuBar, ImageMenuItem, FileFilter};

use std::env::args;
use std::thread;
use std::time::Duration;
use std::path::PathBuf;
use core::borrow::{BorrowMut, Borrow};
use std::cell::RefCell;
use std::rc::Rc;

struct CompressorUI{
    paths: Rc<RefCell<Vec<PathBuf>>>
}

impl CompressorUI{

    fn new()->CompressorUI{
        CompressorUI{
            paths: Rc::new(RefCell::new(Vec::new()))
        }
    }

    fn build_ui(&self,application: &gtk::Application) {
        let glade_src = include_str!("main.glade");

        let builder = Builder::new_from_string(glade_src);

        let window: Window = builder.get_object("wnd_main").expect("Couldn't get window");
        window.set_title("Compressor");
        window.set_application(application);

        let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

        let window_weak = window.downgrade();

        let btn_start: Button = builder.get_object("btn_start").expect("Couldn't get btn_start");
        let start_tx = tx.clone();
        let start_rc = self.paths.clone();
        btn_start.connect_clicked(move |_| {
            handle_start(start_tx.clone(),start_rc.clone())
        });


        let btn_file: Button = builder.get_object("btn_file").expect("Couldn't get btn_file");
        let btn_file_win_ref = window_weak.clone();
        let btn_file_tx = tx.clone();
        let btn_file_rc = self.paths.clone();
        btn_file.connect_clicked( move |_| {
            let window = match btn_file_win_ref.upgrade(){
                Some(w) => w,
                _ => return ()
            };
            file_chooser_dialog(window,btn_file_tx.clone(),btn_file_rc.clone());
        });

        let targets = vec![gtk::TargetEntry::new("text/uri-list", TargetFlags::OTHER_APP, 0)];
        let text_view: TextView = builder.get_object("text_view").expect("Couldn't get text_view");
        text_view.drag_dest_set(DestDefaults::HIGHLIGHT, &targets, DragAction::COPY);
        let text_buffer = text_view.get_buffer().unwrap();
        text_buffer.set_text("\n\n\t\tDrop Files here.");
        let drop_tx = tx.clone();
        let drop_rc = self.paths.clone();
        text_view.connect_drag_data_received(move |w, _, _, _, d, _, _| {
            let files = d.get_uris()
                .iter()
                .map(|gs| Url::parse(&gs))
                .filter(|res|res.is_ok())
                .map(|url| url.unwrap())
                .map(|url| url.to_file_path())
                .filter(|res|res.is_ok())
                .map(|path| path.unwrap())
                .collect::<Vec<_>>();
            handle_added_pathbuffs(files,drop_tx.clone(),drop_rc.clone());
        });

        rx.attach(None, move |text| {
            text_buffer.set_text(&text);
            glib::Continue(true)
        });


        let menu_item_open: ImageMenuItem = builder.get_object("mi_open").expect("Couldn't get text_view");
        let menu_item_open_win_ref = window_weak.clone();
        let menu_item_open_tx = tx.clone();
        let menu_item_open_rc = self.paths.clone();
        menu_item_open.connect_activate( move |_| {
            let window = match menu_item_open_win_ref.upgrade(){
                Some(w) => w,
                _ => return ()
            };
            file_chooser_dialog(window,menu_item_open_tx.clone(),menu_item_open_rc.clone());
        });

        window.show_all();
    }
}

fn handle_added_pathbuffs(mut paths: Vec<PathBuf>,tx: Sender<String>,mut rc: Rc<RefCell<Vec<PathBuf>>>){
    let mut vec =(*rc).borrow_mut();
    vec.clear();
    vec.append(&mut paths);
    let text = vec.iter().map(|p|p.to_str().unwrap().to_owned()+"\n").collect::<Vec<_>>().concat();
    tx.send(text).expect("Couldn't send data to channel");
}


fn file_chooser_dialog(window: Window, tx: Sender<String>,rc: Rc<RefCell<Vec<PathBuf>>>){
    let dialog = FileChooserDialog::new(Some("Choose a file"), Some(&window),
                                        FileChooserAction::Open);
    dialog.add_buttons(&[
        ("Cancel", ResponseType::Cancel.into()),
        ("Open", ResponseType::Ok.into())

    ]);

    let filter = FileFilter::new();
    filter.add_pattern("*.jpg");
    filter.add_pattern("*.jpeg");
    filter.add_pattern("*.JPG");
    dialog.set_filter(&filter);
    dialog.set_select_multiple(true);
    dialog.run();
    let files = dialog.get_filenames();
    dialog.destroy();
    println!("Files: {:?}", files);
    handle_added_pathbuffs(files,tx,rc);
}

fn handle_start(tx: Sender<String>,rc: Rc<RefCell<Vec<PathBuf>>>){
    let rc =(*rc).borrow();

    if rc.len()==0{
        tx.send(String::from("Select files first")).expect("Could not send text");
        return
    }
    else if rc.len()==1 && rc.get(0).unwrap().is_dir(){
        let paths = find_all_jpegs(rc.get(0).unwrap());
        let specs = paths.iter()
            .map(|path|
                getSpec(Quality::Fastest,
                        Scale::Ratio(0.2f32),
                        path.to_str().unwrap(),
                        rc.get(0).unwrap().join("compressed")))
            .collect();
        let num_saved_images = compress_specs(specs);
    }
    else {
        let specs = rc.iter()
            .map(|path|
                getSpec(Quality::Fastest,
                        Scale::Ratio(0.2f32),
                        path.to_str().unwrap(),
                        rc.get(0).unwrap().parent().unwrap().join("compressed")))
            .collect();
        let num_saved_images = compress_specs(specs);
    }
}


fn main() {
    let application = gtk::Application::new("dev.amann.test.rust_gk",
                                            Default::default())
        .expect("Initialization failed...");

    let ui = CompressorUI::new();

    application.connect_activate(move |app| {
        ui.build_ui(app);
    });

    application.run(&args().collect::<Vec<_>>());
}
