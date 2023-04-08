use glib::Bytes;
use gtk::gdk_pixbuf::{Pixbuf, Colorspace};
use gtk::gio::{File, FileCreateFlags};
use gtk::gio::prelude::*;
use gtk::glib::{clone, MainContext, PRIORITY_DEFAULT};
use gtk::{glib, FileDialog, ApplicationWindow, AlertDialog, Button, Application, ListBox};
use gtk::prelude::*;

const APP_ID: &str = "fr.flyingdev.gtk_rust";

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
        Ok([u8::from(self.buffer[y * self.rowstride as usize + x * self.n_channels as usize]), u8::from(self.buffer[y * self.rowstride as usize + x * self.n_channels as usize + 1]), u8::from(self.buffer[y * self.rowstride as usize + x * self.n_channels as usize + 2])])
    }
}

impl From<Pixbuf> for PixelBuffer {
    fn from(value: Pixbuf) -> Self {
        Self { buffer: value.read_pixel_bytes(), n_channels: value.n_channels().try_into().unwrap_or(3), height: value.height().try_into().unwrap(), width: value.width().try_into().unwrap(), rowstride: value.rowstride().try_into().unwrap(), pixbuf: value }
    }
}

fn main() -> glib::ExitCode {
    /*let mut image_hote = PixelBuffer::from(match Pixbuf::from_file("pict.jpg") {
        Ok(val) => {
            val
        }
        Err(msg) => {
            println!("Err: {msg}");
            Pixbuf::new(Colorspace::Rgb, false, 8, 1000, 1000).unwrap()
        }
    });

    let image_invite = PixelBuffer::from(match Pixbuf::from_file("pict2.jpg") {
        Ok(val) => {
            val
        }
        Err(msg) => {
            println!("Err: {msg}");
            Pixbuf::new(Colorspace::Rgb, false, 8, 100, 100).unwrap()
        }
    });

    let mut etat_pixel = 0u8;

    let mut pixel_actif_sur_hote = [0u8; 3];

    let mut hote_x = 0usize;
    let mut hote_y = 0usize;

    for octet in [image_invite.height / 256, image_invite.height % 256, image_invite.width / 256, image_invite.width % 256] {
        println!("[Enc] {}: {:?}", octet, dec_vers_bin(octet));
        for bit in dec_vers_bin(octet) {
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
    let octets_excedentaires = image_invite.rowstride - (image_invite.width * 3);
    let mut passer_octet = false;

    for octet in image_invite.buffer.into_iter() {
        if !passer_octet {
            for bit in dec_vers_bin(usize::from(octet.to_owned())) {
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
                    hote_x = hote_x + 1;
                    if hote_x > image_hote.width - 1 {
                        hote_y += 1;
                        hote_x = 0;
                    }
                }
            }
            compteur_octet += 1;
            if compteur_octet > image_invite.width * 3 {
                passer_octet = true;
                compteur_octet = octets_excedentaires;
            }
        } else {
            compteur_octet -= 1;
            if compteur_octet == 0 {
                passer_octet = false;
                compteur_octet = 1;
            }
        }
    }

    if etat_pixel != 0 {
        image_hote.pixbuf.put_pixel(hote_x.try_into().unwrap(), hote_y.try_into().unwrap(), pixel_actif_sur_hote[0], pixel_actif_sur_hote[1], pixel_actif_sur_hote[2], 0);
    }

    if let Err(msg) = image_hote.pixbuf.savev("errh.png", "png", &[("compression", "0")]) {
        println!("Erreur lors de l'enregistrement:\n{msg}");
    }
    
    drop(image_hote);
    drop(image_invite);

    let image_a_decrypter = PixelBuffer::from(Pixbuf::from_file("errh.png").expect("Erreur lors de l'ouverture"));

    let mut octet_actif = [0u8; 8];
    let mut compteur_bits = 0usize;
    let mut compteur_taille_image = 0usize;
    let mut taille_image = [0u8; 4];
    hote_x = 0;
    hote_y = 0;
    etat_pixel = 0;

    pixel_actif_sur_hote = image_a_decrypter.get_pixel(hote_x, hote_y).expect("Erreur innatendue !");
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
            println!("[Dec] {}: {:?}", bin_vers_dec(octet_actif), octet_actif);
            taille_image[compteur_taille_image] = bin_vers_dec(octet_actif);
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
            pixel_actif_sur_image_decrypte[compteur_pixel_image_decrypte] = bin_vers_dec(octet_actif);
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

    image_decrypte.pixbuf.savev("ohh.png", "png", &[("compression", "0")]).expect("Erreur lors de l'enregistrement !");*/

    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(build_ui);

    app.run()
}

fn build_ui(app: &Application) {
    // Create a button with label and margins
    let container = ListBox::new();

    // Create a window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("My GTK App")
        .child(&container)
        .build();

    let button = Button::builder()
        .label("Encoder")
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    container.append(&button);

    let window_ptr = window.clone();
    button.connect_clicked(move |_| {
        let maincontext = MainContext::default();
        maincontext.spawn_local(clone!(@weak window_ptr => async move {
            match encoder(&window_ptr).await {
                Ok(_) => {}
                Err(msg) => {
                    AlertDialog::builder()
                        .message(msg)
                        .build()
                        .show(Some(&window_ptr));
                }
            }
        }));
        // encoder(&window);
    });

    let bouton2 = Button::builder()
        .label("Décoder")
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    container.append(&bouton2);

    let window_ptr = window.clone();
    bouton2.connect_clicked(move |_| {
            let maincontext = MainContext::default();
            maincontext.spawn_local(clone!(@weak window_ptr => async move {
                decoder(&window_ptr).await;
            }));
        });

    // Present window
    window.present();
}

fn dec_vers_bin(mut nombre: usize) -> [usize; 8] {
    let mut bits = [0usize; 8];
    let mut puissance: usize;

    if nombre > 255 {
        panic!("Erreur inattendue !");
    }

    if nombre == 0 {
        return bits
    }

    for index in 0u32..8u32 {
        puissance = 2usize.pow(7-index);
        if nombre / puissance > 0 {
            bits[index as usize] = nombre / puissance;
            nombre -= puissance;
        }
    }
    bits
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

fn bin_vers_dec(bits: [u8; 8]) -> u8 {
    let mut nombre = 0u8;
    for (compteur, bit) in bits.into_iter().enumerate() {
        nombre += bit * 2u8.pow((7-compteur).try_into().unwrap());
    }
    nombre
}

async fn encoder(fenetre_principale: &ApplicationWindow) -> Result<(), String> {
    let fichier_image_invite = ouvrir_fichier("Ouvrir l'image à encoder", fenetre_principale).await.expect("Erreur lors de l'ouverture du fichier");
    
    let image_hote = PixelBuffer::from(match Pixbuf::from_stream_future(&fichier_image_invite.read_future(PRIORITY_DEFAULT).await.unwrap()).await {
        Ok(val) => {
            val
        }
        Err(msg) => {
            println!("Err: {msg}");
            Pixbuf::new(Colorspace::Rgb, false, 8, 1000, 1000).unwrap()
        }
    });

    let fichier_image_invite = ouvrir_fichier("Ouvrir l'image qui va cacher la première image", fenetre_principale).await.expect("Erreur lors de l'ouverture du fichier");
    let image_invite = PixelBuffer::from(match Pixbuf::from_stream_future(&fichier_image_invite.read_future(PRIORITY_DEFAULT).await.unwrap()).await {
        Ok(val) => {
            val
        }
        Err(msg) => {
            println!("Err: {msg}");
            Pixbuf::new(Colorspace::Rgb, false, 8, 100, 100).unwrap()
        }
    });

    let mut etat_pixel = 0u8;

    let mut pixel_actif_sur_hote = [0u8; 3];

    let mut hote_x = 0usize;
    let mut hote_y = 0usize;

    for octet in [image_invite.height / 256, image_invite.height % 256, image_invite.width / 256, image_invite.width % 256] {
        println!("[Enc] {}: {:?}", octet, dec_vers_bin(octet));
        for bit in dec_vers_bin(octet) {
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
    let octets_excedentaires = image_invite.rowstride - (image_invite.width * 3);
    let mut passer_octet = false;

    for octet in image_invite.buffer.into_iter() {
        if !passer_octet {
            for bit in dec_vers_bin(usize::from(octet.to_owned())) {
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
                    hote_x = hote_x + 1;
                    if hote_x > image_hote.width - 1 {
                        hote_y += 1;
                        hote_x = 0;
                    }
                }
            }
            compteur_octet += 1;
            if compteur_octet > image_invite.width * 3 {
                passer_octet = true;
                compteur_octet = octets_excedentaires;
            }
        } else {
            compteur_octet -= 1;
            if compteur_octet == 0 {
                passer_octet = false;
                compteur_octet = 1;
            }
        }
    }

    if etat_pixel != 0 {
        image_hote.pixbuf.put_pixel(hote_x.try_into().unwrap(), hote_y.try_into().unwrap(), pixel_actif_sur_hote[0], pixel_actif_sur_hote[1], pixel_actif_sur_hote[2], 0);
    }

    /*if let Err(msg) = image_hote.pixbuf.savev("errh.png", "png", &[("compression", "0")]) {
        return Err(format!("Erreur lors de l'enregistrement:\n{msg}"))
    }*/

    let flux_de_sauvegarde = sauvegarder_fichier("Sauvegarder l'image encodée", &fenetre_principale).await.unwrap().create_readwrite_future(FileCreateFlags::NONE, PRIORITY_DEFAULT).await.unwrap();

    if let Err(msg) = image_hote.pixbuf.save_to_streamv_future(&flux_de_sauvegarde.output_stream(), "png", &[("compression", "0")]).await {
        AlertDialog::builder()
            .message(msg.message())
            .build()
            .show(Some(fenetre_principale));
    }

    flux_de_sauvegarde.close_future(PRIORITY_DEFAULT).await.unwrap();

    Ok(())
}

async fn ouvrir_fichier(nom: &str, fenetre_principale: &ApplicationWindow) -> Result<File, String> {
    loop {
        match FileDialog::builder().title(nom).build().open_future(Some(fenetre_principale)).await {
            Ok(file) => {
                return Ok(file)
            }
            Err(msg) => {
                match AlertDialog::builder()
                    .message(format!("Réessayer ?\n {}", msg.message()))
                    .buttons(["Oui", "Non"])
                    .default_button(0)
                    .cancel_button(1)
                    .build()
                    .choose_future(Some(fenetre_principale)).await.unwrap_or(0) {
                    0 => {}
                    1 => {
                        break;
                    }
                    nmb => {
                        panic!("Erreur inattendue n°1 ({nmb})");
                    }
                }
            }
        };
    }
    return Err("Abandon de l'utilisateur".to_owned())
}

async fn sauvegarder_fichier(nom: &str, fenetre_principale: &ApplicationWindow) -> Result<File, String> {
    match FileDialog::builder()
        .title(nom)
        .build()
        .save_future(Some(fenetre_principale))
        .await {
        Ok(fichier) => {
            Ok(fichier)
        }
        Err(msg) => {
            Err(msg.message().to_owned())
        }
    }
}

async fn decoder(fenetre_principale: &ApplicationWindow) {
    let image_a_decrypter = PixelBuffer::from(Pixbuf::from_stream_future(&ouvrir_fichier("Ouvrir le fichier à décoder", fenetre_principale).await.unwrap().read_future(PRIORITY_DEFAULT).await.unwrap()).await.unwrap());

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
            println!("[Dec] {}: {:?}", bin_vers_dec(octet_actif), octet_actif);
            taille_image[compteur_taille_image] = bin_vers_dec(octet_actif);
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
            pixel_actif_sur_image_decrypte[compteur_pixel_image_decrypte] = bin_vers_dec(octet_actif);
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

    let flux_de_sauvegarde = sauvegarder_fichier("Sauvegarder l'image décodée", fenetre_principale).await.unwrap().create_readwrite_future(FileCreateFlags::NONE, PRIORITY_DEFAULT).await.unwrap();
    image_decrypte.pixbuf.save_to_streamv_future(&flux_de_sauvegarde.output_stream(), "png", &[("compression", "0")]).await.unwrap();
    flux_de_sauvegarde.close_future(PRIORITY_DEFAULT).await.unwrap();
}
