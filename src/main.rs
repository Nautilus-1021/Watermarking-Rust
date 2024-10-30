#![windows_subsystem = "windows"]

use gtk::{glib, gdk_pixbuf, gio};

use gdk_pixbuf::Pixbuf;
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
    bouton_encodage.connect_clicked(clone!(#[strong] fenetre, move |_| {
        glib::spawn_future_local(clone!(#[strong] fenetre, async move { callback_encodage(fenetre).await; }));
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

    bouton_decodage.connect_clicked(clone!(#[strong] fenetre, move |_| {
        glib::spawn_future_local(clone!(#[strong] fenetre, async move { callback_decodage(fenetre).await; }));
    }));

    // Afficher la fenêtre
    fenetre.present();
}

async fn callback_encodage(fenêtre: ApplicationWindow) {
    let fichier_image_invitée = match ouvrir_fichier("Ouvrir l'image à encoder", &fenêtre).await {
        Ok(fichier) => {
            fichier
        }
        Err(_) => {
            return;
        }
    };
    let image_invitée_flux = fichier_image_invitée.read_future(Priority::DEFAULT).await.expect("Impossible d'ouvrir le fichier !");
    let image_invitée = PixelBuffer::from(Pixbuf::from_stream_future(&image_invitée_flux).await.expect("Impossible de charger l'image depuis le fichier !"));

    let fichier_image_hôte = match ouvrir_fichier("Ouvrir l'image qui va cacher la première image", &fenêtre).await {
        Ok(fichier) => {
            fichier
        }
        Err(_) => {
            return;
        }
    };
    let image_hôte_flux = fichier_image_hôte.read_future(Priority::DEFAULT).await.expect("Impossible d'ouvrir le fichier !");
    let image_hôte = PixelBuffer::from(Pixbuf::from_stream_future(&image_hôte_flux).await.expect("Impossible de charger l'image depuis le fichier !"));
    
    if image_invitée.width * image_invitée.height + 2 > image_hôte.height * image_hôte.width {
        AlertDialog::builder()
            .message(format!("Le processus à de grande chance de ne pas finir car l'image à cacher est trop grande par rapport à l'image qui cache.\nVeuillez prendre une image d'un minimum de {val}x{val}", val = f64::from((image_invitée.width as u32 * image_invitée.height as u32 + 2) * 8).sqrt().ceil()))
            .build()
            .show(Some(&fenêtre));
    }

    let image_encodee = algo::encoder(image_invitée, image_hôte);

    let flux_de_sauvegarde = match sauvegarder_fichier("Sauvegarder l'image encodée", &fenêtre, true).await {
        Ok(flux) => flux,
        Err(_) => return
    }.create_readwrite_future(FileCreateFlags::NONE, Priority::DEFAULT).await.unwrap();

    if let Err(msg) = image_encodee.pixbuf.save_to_streamv_future(&flux_de_sauvegarde.output_stream(), "png", &[]).await {
        AlertDialog::builder()
            .message(msg.message())
            .build()
            .show(Some(&fenêtre));
    }

    flux_de_sauvegarde.close_future(Priority::DEFAULT).await.unwrap();
}

async fn callback_decodage(fenetre: ApplicationWindow) {
    let fichier_image_à_décrypter = match ouvrir_fichier("Ouvrir le fichier à décoder", &fenetre).await {
        Ok(val) => val,
        Err(_) => return
    };
    let image_à_décrypter_flux = fichier_image_à_décrypter.read_future(Priority::DEFAULT).await.unwrap();
    let image_a_decrypter = PixelBuffer::from(Pixbuf::from_stream_future(&image_à_décrypter_flux).await.unwrap());

    let result = algo::decoder(image_a_decrypter);

    let fichier_sauvegarde = match sauvegarder_fichier("Sauvegarder l'image décodée", &fenetre, false).await {
        Ok(flux) => flux,
        Err(_) => return
    };

    let type_image = fichier_sauvegarde.basename().and_then(|filename| {
        filename.extension().map(|ext| { ext.to_string_lossy().into_owned() })
    }).and_then(|ext| {
        if ["jpeg", "jpg", "png", "tiff", "ico", "bmp"].contains(&ext.as_str()) {
            Some(ext)
        } else {
            None
        }
    }).unwrap_or("png".to_owned());

    let flux_de_sauvegarde = fichier_sauvegarde.create_readwrite_future(FileCreateFlags::NONE, Priority::DEFAULT).await.unwrap();
    result.pixbuf.save_to_streamv_future(&flux_de_sauvegarde.output_stream(), &type_image, &[]).await.unwrap();
    flux_de_sauvegarde.close_future(Priority::DEFAULT).await.unwrap();
}
