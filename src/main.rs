use gtk::gdk_pixbuf::{Pixbuf, Colorspace};
use gtk::gio::FileCreateFlags;
use gtk::glib::{clone, MainContext, PRIORITY_DEFAULT, Bytes};
use gtk::{glib, gio, ApplicationWindow, AlertDialog, Button, Application, Box, Orientation};

use gtk::gio::prelude::*;
use gtk::prelude::*;

mod outils;

const APP_ID: &str = "fr.flyingdev.watermarking";

struct PixelBuffer {
    buffer: Bytes,
    n_channels: usize,
    height: usize,
    width: usize,
    rowstride: usize,
    pixbuf: Pixbuf
}

impl PixelBuffer {
    fn get_pixel(&self, x: usize, y: usize) -> Result<[u8; 3], &str> {
        if x > self.width || y > self.height {
            return Err("Coordinates are out of the buffer !")
        }
        Ok([u8::from(self.buffer[(y * self.rowstride) + (x * self.n_channels)]), u8::from(self.buffer[(y * self.rowstride) + (x * self.n_channels) + 1]), u8::from(self.buffer[(y * self.rowstride) + (x * self.n_channels) + 2])])
    }
}

impl From<Pixbuf> for PixelBuffer {
    fn from(value: Pixbuf) -> Self {
        Self { buffer: value.read_pixel_bytes(), n_channels: value.n_channels().try_into().unwrap_or(3), height: value.height().try_into().unwrap(), width: value.width().try_into().unwrap(), rowstride: value.rowstride().try_into().unwrap(), pixbuf: value }
    }
}

fn main() -> glib::ExitCode {
    gio::resources_register_include!("watermarking.gresource").expect("Erreur lors du chargement des ressources");

    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(build_ui);

    app.run()
}

fn build_ui(app: &Application) {
    // Create a button with label and margins
    let container = Box::builder()
        .homogeneous(true)
        .orientation(Orientation::Vertical)
        .build();

    // Create a window
    let fenetre = ApplicationWindow::builder()
        .application(app)
        .title("Watermarking")
        .child(&container)
        .build();

    /*let widg_icone = Image::builder()
        .resource("/fr/flyingdev/watermarking/icone/256x256/wat.png")
        .build();

    container.append(&widg_icone);*/

    let bouton_encodage = Button::builder()
        .label("Encoder")
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    container.append(&bouton_encodage);

    let window_ptr = fenetre.clone();
    bouton_encodage.connect_clicked(move |_| {
        let maincontext = MainContext::default();
        maincontext.spawn_local(clone!(@strong window_ptr => async move {
            encoder(&window_ptr).await
        }));
        // encoder(&window);
    });

    let bouton_decodage = Button::builder()
        .label("Décoder")
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    container.append(&bouton_decodage);

    let window_ptr = fenetre.clone();
    bouton_decodage.connect_clicked(move |_| {
            let maincontext = MainContext::default();
            maincontext.spawn_local(clone!(@strong window_ptr => async move {
                decoder(&window_ptr).await;
            }));
        });

    // Present window
    fenetre.present();
}

fn modifier_composante(composante: &mut u8, bit: usize) {
    if *composante == 255 {
        *composante = 254;
    }
    if *composante % 2 == 0 && bit == 1 {
        *composante = *composante + 1;
    } else if *composante % 2 == 1 && bit == 0 {
        *composante = *composante + 1;
    }
}

async fn encoder(fenetre_principale: &ApplicationWindow) {
    let fichier_image_invite = match outils::ouvrir_fichier("Ouvrir l'image à encoder", fenetre_principale).await {
        Ok(fichier) => {
            fichier
        }
        Err(_) => {
            return;
        }
    };
    let image_invite = PixelBuffer::from(match Pixbuf::from_stream_future(&fichier_image_invite.read_future(PRIORITY_DEFAULT).await.unwrap()).await {
        Ok(val) => {
            val
        }
        Err(msg) => {
            println!("Err: {msg}");
            Pixbuf::new(Colorspace::Rgb, false, 8, 100, 100).unwrap()
        }
    });

    let fichier_image_hote = match outils::ouvrir_fichier("Ouvrir l'image qui va cacher la première image", fenetre_principale).await {
        Ok(fichier) => {
            fichier
        }
        Err(_) => {
            return;
        }
    };
    
    let image_hote = PixelBuffer::from(match Pixbuf::from_stream_future(&fichier_image_hote.read_future(PRIORITY_DEFAULT).await.unwrap()).await {
        Ok(val) => {
            val
        }
        Err(msg) => {
            println!("Err: {msg}");
            Pixbuf::new(Colorspace::Rgb, false, 8, 1000, 1000).unwrap()
        }
    });

    if image_invite.width * image_invite.height + 2 > image_hote.height * image_hote.width {
        AlertDialog::builder()
            .message(format!("Le processus à de grande chance de ne pas finir car l'image à cacher est trop grande par rapport à l'image qui cache.\nVeuillez prendre une image d'un minimum de {val}x{val}", val = f64::from((image_invite.width as u32 * image_invite.height as u32 + 2) * 8).sqrt().ceil()))
            .build()
            .show(Some(fenetre_principale));
    }

    let mut etat_pixel = 0u8;

    let mut pixel_actif_sur_hote = [0u8; 3];

    let mut hote_x = 0usize;
    let mut hote_y = 0usize;

    for octet in [image_invite.height / 256, image_invite.height % 256, image_invite.width / 256, image_invite.width % 256] {
        println!("[Enc] {}: {:?}", octet, outils::dec_vers_bin(octet));
        for bit in outils::dec_vers_bin(octet) {
            if etat_pixel == 0 {
                pixel_actif_sur_hote = image_hote.get_pixel(hote_x, hote_y).unwrap();
                modifier_composante(&mut pixel_actif_sur_hote[0], bit);
                etat_pixel += 1;
            } else if etat_pixel == 1 {
                modifier_composante(&mut pixel_actif_sur_hote[1], bit);
                etat_pixel += 1;
            } else if etat_pixel == 2 {
                modifier_composante(&mut pixel_actif_sur_hote[2], bit);
                etat_pixel = 0;
                image_hote.pixbuf.put_pixel(hote_x.try_into().unwrap(), hote_y.try_into().unwrap(), pixel_actif_sur_hote[0], pixel_actif_sur_hote[1], pixel_actif_sur_hote[2], 0);
                hote_x += 1;
                if hote_x > image_hote.width - 1 {
                    hote_y += 1;
                    hote_x = 0;
                }
            }
        }
    }

    let mut compteur_octet = 1usize;
    let mut compteur_octet2 = 1usize;
    let octets_excedentaires = image_invite.rowstride - (image_invite.width * image_invite.n_channels);
    let mut passer_octet = false;
    let mut passer_octet2 = false;

    if image_invite.n_channels > 3 && image_invite.width * image_invite.height == (image_invite.buffer.len() - (octets_excedentaires * image_invite.height)) / image_invite.n_channels {
        println!("Test passé !");
    }

    if image_invite.n_channels > 3 && image_invite.rowstride == (image_invite.width * image_invite.n_channels) + octets_excedentaires {
        println!("Test n°2 passé !");
    }

    for octet in image_invite.buffer.into_iter() {
        if !passer_octet && !passer_octet2 {
            for bit in outils::dec_vers_bin(usize::from(octet.to_owned())) {
                if etat_pixel == 0 {
                    pixel_actif_sur_hote = image_hote.get_pixel(hote_x, hote_y).unwrap();
                    modifier_composante(&mut pixel_actif_sur_hote[0], bit);
                    etat_pixel += 1;
                } else if etat_pixel == 1 {
                    modifier_composante(&mut pixel_actif_sur_hote[1], bit);
                    etat_pixel += 1;
                } else if etat_pixel == 2 {
                    modifier_composante(&mut pixel_actif_sur_hote[2], bit);
                    etat_pixel = 0;
                    image_hote.pixbuf.put_pixel(hote_x.try_into().unwrap(), hote_y.try_into().unwrap(), pixel_actif_sur_hote[0], pixel_actif_sur_hote[1], pixel_actif_sur_hote[2], 0);
                    hote_x += 1;
                    if hote_x > image_hote.width - 1 {
                        hote_y += 1;
                        hote_x = 0;
                    }
                }
            }
            compteur_octet += 1;
            compteur_octet2 += 1;
            if compteur_octet > image_invite.width * image_invite.n_channels {
                if octets_excedentaires > 0 {
                    passer_octet = true;
                    compteur_octet = octets_excedentaires;
                } else {
                    compteur_octet = 1;
                }
            } else if compteur_octet2 > 3 {
                passer_octet2 = true;
                compteur_octet2 = image_invite.n_channels - 3;
            }
        } else if passer_octet {
            compteur_octet -= 1;
            if compteur_octet == 0 {
                passer_octet = false;
                compteur_octet = 1;
                compteur_octet2 = 1;
            }
        } else if passer_octet2 {
            compteur_octet2 -= 1;
            compteur_octet += 1;
            if compteur_octet2 <= 0 {
                passer_octet2 = false;
                compteur_octet2 = 1;
            }
        }
    }

    if etat_pixel != 0 {
        image_hote.pixbuf.put_pixel(hote_x.try_into().unwrap(), hote_y.try_into().unwrap(), pixel_actif_sur_hote[0], pixel_actif_sur_hote[1], pixel_actif_sur_hote[2], 0);
    }

    let flux_de_sauvegarde = match outils::sauvegarder_fichier("Sauvegarder l'image encodée", &fenetre_principale).await {
        Ok(flux) => {
            flux
        }
        Err(_) => {
            return;
        }
    }.create_readwrite_future(FileCreateFlags::NONE, PRIORITY_DEFAULT).await.unwrap();

    if let Err(msg) = image_hote.pixbuf.save_to_streamv_future(&flux_de_sauvegarde.output_stream(), "png", &[("compression", "0")]).await {
        AlertDialog::builder()
            .message(msg.message())
            .build()
            .show(Some(fenetre_principale));
    }

    flux_de_sauvegarde.close_future(PRIORITY_DEFAULT).await.unwrap();
}

async fn decoder(fenetre_principale: &ApplicationWindow) {
    let image_a_decrypter = PixelBuffer::from(Pixbuf::from_stream_future(&outils::ouvrir_fichier("Ouvrir le fichier à décoder", fenetre_principale).await.unwrap().read_future(PRIORITY_DEFAULT).await.unwrap()).await.unwrap());

    let mut octet_actif = [0u8; 8];
    let mut compteur_bits = 0usize;
    let mut compteur_taille_image = 0usize;
    let mut taille_image = [0u8; 4];
    let mut hote_x = 0;
    let mut hote_y = 0;
    let mut etat_pixel = 0;

    let mut pixel_actif_sur_hote = image_a_decrypter.get_pixel(hote_x, hote_y).expect("Erreur innatendue !");
    hote_x += 1;
    loop {
        if etat_pixel == 0 {
            if pixel_actif_sur_hote[0] % 2 == 0 {
                octet_actif[compteur_bits] = 0;
            } else {
                octet_actif[compteur_bits] = 1;
            }
            compteur_bits += 1;
            etat_pixel += 1;
        } else if etat_pixel == 1 {
            if pixel_actif_sur_hote[1] % 2 == 0 {
                octet_actif[compteur_bits] = 0;
            } else {
                octet_actif[compteur_bits] = 1;
            }
            compteur_bits += 1;
            etat_pixel += 1;
        } else if etat_pixel == 2 {
            if pixel_actif_sur_hote[2] % 2 == 0 {
                octet_actif[compteur_bits] = 0;
            } else {
                octet_actif[compteur_bits] = 1;
            }
            compteur_bits += 1;
            pixel_actif_sur_hote = image_a_decrypter.get_pixel(hote_x, hote_y).expect("Erreur innatendue !");
            hote_x += 1;
            if hote_x > image_a_decrypter.width - 1 {
                hote_y += 1;
                hote_x = 0;
            }
            etat_pixel = 0;
        }

        if compteur_bits == 8 {
            println!("[Dec] {}: {:?}", outils::bin_vers_dec(octet_actif), octet_actif);
            taille_image[compteur_taille_image] = outils::bin_vers_dec(octet_actif);
            compteur_taille_image += 1;
            compteur_bits = 0;
            if compteur_taille_image == 4 {
                break;
            }
        }
    }

    let decrypte_width = i32::from((u16::from(taille_image[2]) * 256) + u16::from(taille_image[3]));
    let decrypte_height = i32::from((u16::from(taille_image[0]) * 256) + u16::from(taille_image[1]));

    let image_decrypte = PixelBuffer::from(Pixbuf::new(Colorspace::Rgb, false, 8, decrypte_width, decrypte_height).unwrap());
    let mut decrypte_x = 0usize;
    let mut decrypte_y = 0usize;
    let mut pixel_actif_sur_image_decrypte = [0u8; 3];
    let mut compteur_pixel_image_decrypte = 0usize;

    loop {
        if etat_pixel == 0 {
            if pixel_actif_sur_hote[0] % 2 == 0 {
                octet_actif[compteur_bits] = 0;
            } else {
                octet_actif[compteur_bits] = 1;
            }
            compteur_bits += 1;
            etat_pixel += 1;
        } else if etat_pixel == 1 {
            if pixel_actif_sur_hote[1] % 2 == 0 {
                octet_actif[compteur_bits] = 0;
            } else {
                octet_actif[compteur_bits] = 1;
            }
            compteur_bits += 1;
            etat_pixel += 1;
        } else if etat_pixel == 2 {
            if pixel_actif_sur_hote[2] % 2 == 0 {
                octet_actif[compteur_bits] = 0;
            } else {
                octet_actif[compteur_bits] = 1;
            }
            compteur_bits += 1;
            pixel_actif_sur_hote = image_a_decrypter.get_pixel(hote_x, hote_y).expect("Erreur innatendue !");
            hote_x += 1;
            if hote_x > image_a_decrypter.width - 1 {
                hote_y += 1;
                hote_x = 0;
            }
            etat_pixel = 0;
        }

        if compteur_bits == 8 {
            pixel_actif_sur_image_decrypte[compteur_pixel_image_decrypte] = outils::bin_vers_dec(octet_actif);
            compteur_pixel_image_decrypte += 1;
            compteur_bits = 0;
            if compteur_pixel_image_decrypte == 3 {
                image_decrypte.pixbuf.put_pixel(decrypte_x.try_into().unwrap(), decrypte_y.try_into().unwrap(), pixel_actif_sur_image_decrypte[0], pixel_actif_sur_image_decrypte[1], pixel_actif_sur_image_decrypte[2], 0);
                decrypte_x += 1;
                compteur_pixel_image_decrypte = 0;
                if decrypte_x > image_decrypte.width - 1 {
                    decrypte_y += 1;
                    decrypte_x = 0;
                    if decrypte_y >= image_decrypte.height - 1 {
                        break;
                    }
                }
            }
        }
    }

    let flux_de_sauvegarde = match outils::sauvegarder_fichier("Sauvegarder l'image décodée", fenetre_principale).await {
        Ok(flux) => {
            flux
        }
        Err(_) => {
            return;
        }
    }.create_readwrite_future(FileCreateFlags::NONE, PRIORITY_DEFAULT).await.unwrap();
    image_decrypte.pixbuf.save_to_streamv_future(&flux_de_sauvegarde.output_stream(), "png", &[("compression", "0")]).await.unwrap();
    flux_de_sauvegarde.close_future(PRIORITY_DEFAULT).await.unwrap();
}
