use gtk::glib::{Error, FileError};
use gtk::{ApplicationWindow, FileFilter, FilterListModel, FileDialog, AlertDialog};
use gtk::gio::File;

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
    filtre.set_name(Some("Fichier image"));

    let _filtres = FilterListModel::builder()
        .filter(&filtre)
        .build();

    loop {
        match FileDialog::builder()
            .title(nom)
            //.filters(&filtres)
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

pub async fn sauvegarder_fichier(nom: &str, fenetre_principale: &ApplicationWindow) -> Result<File, Error> {
    let filtre = FileFilter::new();
    filtre.add_suffix("png");

    let filtres = FilterListModel::builder()
        .filter(&filtre)
        .build();

    match FileDialog::builder()
        .title(nom)
        .filters(&filtres)
        .build()
        .save_future(Some(fenetre_principale))
        .await {
        Ok(fichier) => {
            Ok(fichier)
        }
        Err(msg) => {
            Err(msg)
        }
    }
}
