#![windows_subsystem = "windows"]
extern crate lazy_static;
extern crate gdk;
extern crate gio;
extern crate gtk;
extern crate glib;
extern crate url;
extern crate compressor;
use compressor::{Scale, Quality, compress_specs, find_all_jpegs,get_spec};

use gio::prelude::*;
use gtk::prelude::*;
use url::Url;
use glib::Sender;

use gdk::DragAction;


use gtk::{Window, Builder, Button, FileChooserDialog, FileChooserAction, ResponseType, TextView, DestDefaults, TargetFlags, ImageMenuItem, FileFilter};

use std::env::args;
use std::path::PathBuf;
use std::cell::RefCell;
use std::rc::Rc;

/**
Struct to hold a Vec of the files to be processed
*/
struct CompressorUI{
    paths: Rc<RefCell<Vec<PathBuf>>>
}

impl CompressorUI{

    /**
    Default constructor initializing the struct with an empty vector
    */
    fn new()->CompressorUI{
        CompressorUI{
            paths: Rc::new(RefCell::new(Vec::new()))
        }
    }

    /**
    Create the UI from the glade file and wire up actions
    This is somewhat similar to setup code used in swing
    */
    fn build_ui(&self,application: &gtk::Application) {
        // include the glade file (at compile time)
        let glade_src = include_str!("main.glade");

        // this builder provides access to all components of the defined ui
        let builder = Builder::new_from_string(glade_src);

        let window: Window = builder.get_object("wnd_main").expect("Couldn't get window");
        window.set_title("Compressor");
        window.set_application(application);

        // this channel is for updating the main text view
        let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

        // create a movable, cloneable weak reference
        let window_weak = window.downgrade();


        // The following codeblocks refer to an UI element each
        // TODO Use macros for cloning. In hindsight it was not the brightest decision to do it manually

        // the start button, starts the compression process
        let btn_start: Button = builder.get_object("btn_start").expect("Couldn't get btn_start");
        let start_tx = tx.clone();
        let start_rc = self.paths.clone();
        btn_start.connect_clicked(move |_| {
            handle_start(start_tx.clone(),start_rc.clone())
        });


        // the file button, opens the file chooser dialog
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


        // the quit menu item. It destroys the window and thereby hands over the control flow back to the main function
        let menu_item_quit: ImageMenuItem = builder.get_object("mi_quit").expect("Couldn't get mi_quit");
        let menu_item_quit_win_ref = window_weak.clone();
        menu_item_quit.connect_activate( move |_| {
            let window = match menu_item_quit_win_ref.upgrade(){
                Some(w) => w,
                _ => return ()
            };
            window.destroy();
        });


        // The main text view. This field accepts dropped elements
        let targets = vec![gtk::TargetEntry::new("text/uri-list", TargetFlags::OTHER_APP, 0)];
        let text_view: TextView = builder.get_object("text_view").expect("Couldn't get text_view");
        text_view.drag_dest_set(DestDefaults::HIGHLIGHT, &targets, DragAction::COPY);
        // the text buffer used in the channel to update the text
        let text_buffer = text_view.get_buffer().unwrap();
        // botchy way to place the text, for now this is ok
        text_buffer.set_text("\n\n\t\tDrop Files here.");
        let drop_tx = tx.clone();
        let drop_rc = self.paths.clone();
        // this closures handles dropped elements: the valid uris get mapped to path buffs and then are handed to handler
        text_view.connect_drag_data_received(move |_, _, _, _, d, _, _| {
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
        // this closure is responsible to update the text of the text field according to strings received via the channel
        rx.attach(None, move |text| {
            text_buffer.set_text(&text);
            glib::Continue(true)
        });

        // the open menu item that does open the file chooser dialog (For people that prefer the menu over the button)
        let menu_item_open: ImageMenuItem = builder.get_object("mi_open").expect("Couldn't get mi_open");
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

        // show all components of the window
        window.show_all();
    }
}

/**
This function gets called when ever new paths have been added, either via drag and drop or via the open button or menu item
*/
fn handle_added_pathbuffs(mut paths: Vec<PathBuf>,tx: Sender<String>, rc: Rc<RefCell<Vec<PathBuf>>>){
    // get the cell with internal mutability, since we need to change the content of the vector
    let mut vec =(*rc).borrow_mut();
    //clear and append all
    vec.clear();
    vec.append(&mut paths);
    // generate a multiline string to display on the file
    let text = vec.iter().map(|p|p.to_str().unwrap().to_owned()+"\n").collect::<Vec<_>>().concat();
    tx.send(text).expect("Couldn't send data to channel");
}

/**
Show a file chosser dialog with a filter for jpegs and add the resulting files to the provided vector
*/
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
    // this shows the dialog and blocks execution
    dialog.run();

    // at this point the user has closed the dialog and we take the selected files
    let files = dialog.get_filenames();
    dialog.destroy();
    println!("Files: {:?}", files);
    handle_added_pathbuffs(files,tx,rc);
}

// TODO read the parameters from the settings rather than hardcode them. Refer to the companion crate
/**
Handle a press on the start button,
This checks if there are files available, else shows a warning.
If files are present, the files get compressed depending on the path.
If it is directory, use the childern, else use the files directly.
*/
fn handle_start(tx: Sender<String>,rc: Rc<RefCell<Vec<PathBuf>>>){
    // Get the cell without internal mutability
    let rc =(*rc).borrow();

    // no files -> no compression
    if rc.len()==0{
        tx.send(String::from("Select files first")).expect("Could not send text");
        return
    }
    // only one file that is a dir, use its children
    else if rc.len()==1 && rc.get(0).unwrap().is_dir(){
        let paths = find_all_jpegs(rc.get(0).unwrap());
        let specs = paths.iter()
            .map(|path|
                get_spec(Quality::Fastest,
                        Scale::Ratio(0.2f32),
                        path.clone(),
                        rc.get(0).unwrap().join("compressed")))
            .collect();
        let num_saved_images = compress_specs(specs);
        tx.send(String::from(format!("Processed {} images.",num_saved_images))).expect("Could not send text");
    }
    // several files provided or provided file is not a directory, use them directly
    else {
        let specs = rc.iter()
            .map(|path|
                get_spec(Quality::Fastest,
                        Scale::Ratio(0.2f32),
                        path.clone(),
                        rc.get(0).unwrap().parent().unwrap().join("compressed")))
            .collect();
        let num_saved_images = compress_specs(specs);
        tx.send(String::from(format!("Processed {} images.",num_saved_images))).expect("Could not send text");
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

    // This is marked as an error in IntelliJ ?
    application.run(&args().collect::<Vec<_>>());
}
