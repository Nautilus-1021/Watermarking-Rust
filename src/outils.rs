use gtk::{gio, glib};

use glib::{Error, FileError};
use gio::{File, ListStore};
use gtk::{ApplicationWindow, FileFilter, FileDialog, AlertDialog};

pub fn bin_vers_dec(bits: [u8; 8]) -> u8 {
    let mut nombre = 0u8;
    for (compteur, bit) in bits.into_iter().enumerate() {
        nombre += bit * 2u8.pow((7-compteur).try_into().unwrap());
    }
    nombre
}

pub fn dec_vers_bin(mut nombre: usize) -> [usize; 8] {
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

pub async fn ouvrir_fichier(nom: &str, fenetre_principale: &ApplicationWindow) -> Result<File, Error> {
    let filtre = FileFilter::new();
    filtre.add_pixbuf_formats();

    let filtres = ListStore::new::<FileFilter>();
    filtres.append(&filtre);

    loop {
        match FileDialog::builder()
            .title(nom)
            .filters(&filtres)
            .default_filter(&filtre)
            .build()
            .open_future(Some(fenetre_principale))
            .await {
            Ok(file) => {
                return Ok(file)
            }
            Err(_) => {
                match AlertDialog::builder()
                    .message("Réessayer ?")
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
    return Err(Error::new(FileError::Failed, "Abandon de l'utilisateur"))
}

pub async fn sauvegarder_fichier(nom: &str, fenetre_principale: &ApplicationWindow, restreindre_png: bool) -> Result<File, Error> {
    let filtres = ListStore::new::<FileFilter>();

    let filtre = FileFilter::new();
    filtre.add_suffix("png");
    filtre.set_name(Some("Images PNG"));

    filtres.append(&filtre);

    if !restreindre_png {
        let filtre_rien = FileFilter::new();
        filtre_rien.add_pattern("*.*");
        filtre_rien.set_name(Some("Tout les fichiers"));

        filtres.append(&filtre_rien);
    }

    match FileDialog::builder()
        .title(nom)
        .filters(&filtres)
        .build()
        .save_future(Some(fenetre_principale))
        .await {
        Ok(file) => {
            Ok(file)
        }
        Err(msg) => {
            Err(msg)
        }
    }
}

pub fn modifier_composante(composante: &mut u8, bit: usize) {
    if *composante == 255 {
        *composante = 254;
    }
    if *composante % 2 == 0 && bit == 1 {
        *composante = *composante + 1;
    } else if *composante % 2 == 1 && bit == 0 {
        *composante = *composante + 1;
    }
}
