use gtk::gdk_pixbuf::{Pixbuf, Colorspace};
use gtk::glib::Bytes;

use crate::outils::{dec_vers_bin, bin_vers_dec, modifier_composante};

#[derive(Clone)]
pub struct PixelBuffer {
    buffer: Bytes,
    n_channels: usize,
    pub height: usize,
    pub width: usize,
    rowstride: usize,
    pub pixbuf: Pixbuf
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

pub async fn encoder(image_invite: PixelBuffer, image_hote: PixelBuffer) -> PixelBuffer {
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
            } else if compteur_octet2 > 3 && image_invite.n_channels > 3 {
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

    image_hote
}

pub async fn decoder(image_a_decrypter: PixelBuffer) -> PixelBuffer {
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

    image_decrypte
}
