#![windows_subsystem = "windows"]

use gtk::{glib, gdk_pixbuf, gio};

use gdk_pixbuf::{Pixbuf, Colorspace};
use gio::FileCreateFlags;
use glib::{clone, Priority};
use gtk::{ApplicationWindow, AlertDialog, Button, Application, Box, Orientation};

use gio::prelude::*;
use gtk::prelude::*;

mod outils;
mod algo;

use algo::PixelBuffer;
use outils::{ouvrir_fichier, sauvegarder_fichier};

const APP_ID: &str = "fr.flyingdev.watermarking";

fn main() -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(build_ui);

    app.run()
}

fn build_ui(app: &Application) {
    // Création du conteneur principal de l'interface
    let container = Box::builder()
        .homogeneous(true)
        .orientation(Orientation::Vertical)
        .build();

    // Création de la fenêtre
    let fenetre = ApplicationWindow::builder()
        .application(app)
        .title("Watermarking")
        .child(&container)
        .build();

    // Création d'un bouton
    let bouton_encodage = Button::builder()
        .label("Encoder")
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    container.append(&bouton_encodage);

    // Ajout d'un callback pour quand le bouton sera cliqué
    bouton_encodage.connect_clicked(clone!(@strong fenetre, @strong app => move |_| {
        glib::spawn_future_local(clone!(@strong fenetre, @strong app => async move {
            let fichier_image_invite = match ouvrir_fichier("Ouvrir l'image à encoder", &fenetre).await {
                Ok(fichier) => {
                    fichier
                }
                Err(_) => {
                    return;
                }
            };
            let image_invite = PixelBuffer::from(Pixbuf::from_stream_future(&fichier_image_invite.read_future(Priority::DEFAULT).await.unwrap()).await.unwrap_or(Pixbuf::new(Colorspace::Rgb, false, 8, 100, 100).unwrap()));
        
            let fichier_image_hote = match ouvrir_fichier("Ouvrir l'image qui va cacher la première image", &fenetre).await {
                Ok(fichier) => {
                    fichier
                }
                Err(_) => {
                    return;
                }
            };
            let image_hote = PixelBuffer::from(Pixbuf::from_stream_future(&fichier_image_hote.read_future(Priority::DEFAULT).await.unwrap()).await.unwrap_or(Pixbuf::new(Colorspace::Rgb, false, 8, 1000, 1000).unwrap()));
            
            if image_invite.width * image_invite.height + 2 > image_hote.height * image_hote.width {
                AlertDialog::builder()
                    .message(format!("Le processus à de grande chance de ne pas finir car l'image à cacher est trop grande par rapport à l'image qui cache.\nVeuillez prendre une image d'un minimum de {val}x{val}", val = f64::from((image_invite.width as u32 * image_invite.height as u32 + 2) * 8).sqrt().ceil()))
                    .build()
                    .show(Some(&fenetre));
            }

            let image_encodee = algo::encoder(image_invite, image_hote).await;

            let flux_de_sauvegarde = match sauvegarder_fichier("Sauvegarder l'image encodée", &fenetre, true).await {
                Ok(flux) => {
                    flux
                }
                Err(_) => {
                    return;
                }
            }.create_readwrite_future(FileCreateFlags::NONE, Priority::DEFAULT).await.unwrap();
        
            if let Err(msg) = image_encodee.pixbuf.save_to_streamv_future(&flux_de_sauvegarde.output_stream(), "png", &[]).await {
                AlertDialog::builder()
                    .message(msg.message())
                    .build()
                    .show(Some(&fenetre));
            }
        
            flux_de_sauvegarde.close_future(Priority::DEFAULT).await.unwrap();
        }));
    }));

    // Même chose avec le bouton pour décoder
    let bouton_decodage = Button::builder()
        .label("Décoder")
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    container.append(&bouton_decodage);

    bouton_decodage.connect_clicked(clone!(@strong fenetre => move |_| {
            glib::spawn_future_local(clone!(@strong fenetre => async move {
                let image_a_decrypter = PixelBuffer::from(Pixbuf::from_stream_future(&ouvrir_fichier("Ouvrir le fichier à décoder", &fenetre).await.unwrap().read_future(Priority::DEFAULT).await.unwrap()).await.unwrap());

                let result = algo::decoder(image_a_decrypter).await;

                let fichier_sauvegarde = match sauvegarder_fichier("Sauvegarder l'image décodée", &fenetre, false).await {
                    Ok(flux) => flux,
                    Err(_) => return
                };

                let mut type_image = "png".to_owned();

                match fichier_sauvegarde.basename() {
                    Some(nom) => {
                        match nom.extension() {
                            Some(ext) => {
                                let ext = ext.to_owned();
                                if ["jpeg", "jpg", "png", "tiff", "ico", "bmp"].contains(&ext.to_str().unwrap()) {
                                    type_image = ext.into_string().unwrap();
                                }
                            }
                            None => {}
                        }
                    }
                    None => {}
                }

                let flux_de_sauvegarde = fichier_sauvegarde.create_readwrite_future(FileCreateFlags::NONE, Priority::DEFAULT).await.unwrap();
                result.pixbuf.save_to_streamv_future(&flux_de_sauvegarde.output_stream(), &type_image, &[]).await.unwrap();
                flux_de_sauvegarde.close_future(Priority::DEFAULT).await.unwrap();
            }));
        }));

    // Afficher la fenêtre
    fenetre.present();
}
