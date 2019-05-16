mkdir ./release
cp target/x86_64-pc-windows-gnu/release/*.exe ./release
cp $GTK_INSTALL_PATH/bin/*.dll ./release
mkdir -p ./release/share/glib-2.0/schemas
mkdir ./release/share/icons
cp $GTK_INSTALL_PATH/share/glib-2.0/schemas/* ./release/share/glib-2.0/schemas
cp -r $GTK_INSTALL_PATH/share/icons/* ./release/share/icons
mkdir release/share/gtk-3.0
mkdir release/share/themes
wget https://github.com/B00merang-Project/Windows-10/archive/2.1.zip
unzip 2.1.zip
mv Windows-10-2.1 release/share/themes/Windows10
mv win_theme_settings.ini release/share/gtk-3.0/settings.ini
