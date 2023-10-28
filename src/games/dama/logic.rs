use super::super::util::Player;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, DuplexStream, WriteHalf};
use tokio::time::{timeout, Duration};

pub(crate) async fn partita_in_corso(
    damiera: Vec<Vec<&str>>,
    giocatore: &mut Vec<Player>,
    spectators: &mut WriteHalf<DuplexStream>,
    turno_binaco: bool,
) -> bool {
    let mut fine_partita: bool = false;
    let mut n_pedine_bianche: usize = 0;
    let mut n_pedine_nere: usize = 0;

    // Conto le pedine rimaste a ogni giocatore
    for r in damiera.clone() {
        for c in r {
            if c == "b" || c == "B" {
                n_pedine_bianche += 1;
            } else if c == "n" || c == "N" {
                n_pedine_nere += 1;
            }
        }
    }

    // Se un giocatore esaurisce le pedine o le mossa fermo il gioco
    if n_pedine_bianche == 0 {
        _ = giocatore[0]
            .output
            .write(("Non hai più pedine disponibili.\nI neri vincono la partita!\n\n").as_bytes())
            .await;
        _ = giocatore[1]
            .output
            .write(("I neri vincono la partita!\n\n").as_bytes())
            .await;
        _ = spectators
            .write(("I neri vincono la partita!\n\n").as_bytes())
            .await;

        fine_partita = true;
    } else if n_pedine_nere == 0 {
        _ = giocatore[0]
            .output
            .write(("I bianchi vincono la partita!\n\n").as_bytes())
            .await;
        _ = giocatore[1]
            .output
            .write(
                ("Non hai più pedine disponibili.\nI bianchi vincono la partita!\n\n").as_bytes(),
            )
            .await;
        _ = spectators
            .write(("I bianchi vincono la partita!\n\n").as_bytes())
            .await;

        fine_partita = true;
    } else if mosse_possibili(damiera.clone(), turno_binaco).await == false
    // Controllo se è possibile per il prossimo giocatore fare uno spostamento o una cattura
    {
        if turno_binaco == true {
            _ = giocatore[0]
                .output
                .write(
                    ("Non hai più mosse disponibili.\nI neri vincono la partita!\n\n").as_bytes(),
                )
                .await;
            _ = giocatore[1]
                .output
                .write(
                    ("I bianchi non hanno più mosse disponibili.\nI neri vincono la partita!\n\n")
                        .as_bytes(),
                )
                .await;
            _ = spectators
                .write(
                    ("I bianchi non hanno più mosse disponibili.\nI neri vincono la partita!\n\n")
                        .as_bytes(),
                )
                .await;
        } else {
            _ = giocatore[0]
                .output
                .write(
                    ("I neri non hanno più mosse disponibili.\nI bianchi vincono la partita!\n\n")
                        .as_bytes(),
                )
                .await;
            _ = giocatore[1]
                .output
                .write(
                    ("Non hai più mosse disponibili.\nI bianchi vincono la partita!\n\n")
                        .as_bytes(),
                )
                .await;
            _ = spectators
                .write(
                    ("I neri non hanno più mosse disponibili.\nI bianchi vincono la partita!\n\n")
                        .as_bytes(),
                )
                .await;
        }

        fine_partita = true;
    }

    fine_partita
}

async fn mosse_possibili(damiera: Vec<Vec<&str>>, turno_binaco: bool) -> bool {
    let mut mossa: bool = false; // Setto se è possibile o meno fare una mossa che sia uno spostamento o una cattura
    let mut pedine: Vec<Vec<usize>> = Vec::new(); // Setto il vettore che contiene le pedine del giocatore
    let mut dame: Vec<Vec<usize>> = Vec::new(); // Setto il vettore che contiene le dame del giocatore
    let mut row: usize;
    let mut col: usize;

    if turno_binaco == true {
        // Prelevo le Dame e le pedine bianche
        for r in 0..damiera.len() {
            for c in 0..damiera[r].len() {
                if damiera[r][c] == "b" {
                    pedine.push(vec![r, c]);
                } else if damiera[r][c] == "B" {
                    dame.push(vec![r, c]);
                }
            }
        }

        // Controllo se le pedine bianche possono muovere o mangiare
        if pedine.len() > 0 {
            for n in 0..pedine.len() {
                row = pedine[n][0];
                col = pedine[n][1];

                // Controllo gli spostamenti
                if ((row as i32) - 1 >= 0 && (col as i32) - 1 >= 0)
                    && damiera[row - 1][col - 1] == " "
                {
                    // Spostamento SX su
                    mossa = true;
                } else if ((row as i32) - 1 >= 0 && (col as i32) + 1 <= 7)
                    && damiera[row - 1][col + 1] == " "
                {
                    // Spostamento DX su
                    mossa = true;
                }

                // Controllo le catture
                if ((row as i32) - 2 >= 0 && (col as i32) - 2 >= 0)
                    && damiera[row - 1][col - 1] == "n"
                    && damiera[row - 2][col - 2] == " "
                {
                    // Cattura SX su
                    mossa = true;
                } else if ((row as i32) - 2 >= 0 && (col as i32) + 2 <= 7)
                    && damiera[row - 1][col + 1] == "n"
                    && damiera[row - 2][col + 2] == " "
                {
                    // Cattura verso DX su
                    mossa = true;
                }
            }
        }

        // Controllo se le dame bianche possono muovere o mangiare
        if dame.len() > 0 {
            for n in 0..dame.len() {
                row = dame[n][0];
                col = dame[n][1];

                // Controllo gli spostamenti
                if ((row as i32) - 1 >= 0 && (col as i32) - 1 >= 0)
                    && damiera[row - 1][col - 1] == " "
                {
                    // Spostamento SX su
                    mossa = true;
                } else if ((row as i32) - 1 >= 0 && (col as i32) + 1 <= 7)
                    && damiera[row - 1][col + 1] == " "
                {
                    // Spostamento DX su
                    mossa = true;
                }
                if ((row as i32) + 1 <= 7 && (col as i32) + 1 <= 7)
                    && damiera[row + 1][col + 1] == " "
                {
                    // Spostamento DX giù
                    mossa = true;
                } else if ((row as i32) + 1 <= 0 && (col as i32) - 1 >= 0)
                    && damiera[row + 1][col - 1] == " "
                {
                    // Spostamento SX giù
                    mossa = true;
                }

                // Controllo le catture
                if ((row as i32) - 2 >= 0 && (col as i32) - 2 >= 0)
                    && (damiera[row - 1][col - 1] == "n" || damiera[row - 1][col - 1] == "N")
                    && damiera[row - 2][col - 2] == " "
                {
                    // Catturo verso SX su
                    mossa = true;
                } else if ((row as i32) - 2 >= 0 && (col as i32) + 2 <= 7)
                    && (damiera[row - 1][col + 1] == "n" || damiera[row - 1][col + 1] == "N")
                    && damiera[row - 2][col + 2] == " "
                {
                    // Catturo verso DX su
                    mossa = true;
                } else if ((row as i32) + 2 <= 7 && (col as i32) + 2 <= 7)
                    && (damiera[row + 1][col + 1] == "n" || damiera[row + 1][col + 1] == "N")
                    && damiera[row + 2][col + 2] == " "
                {
                    // Catturo verso DX giù
                    mossa = true;
                } else if ((row as i32) + 2 <= 7 && (col as i32) - 2 >= 0)
                    && (damiera[row + 1][col - 1] == "n" || damiera[row + 1][col - 1] == "N")
                    && damiera[row + 1][col - 1] == " "
                {
                    // Catturo verso SX giù
                    mossa = true;
                }
            }
        }
    } else {
        // Prelevo le Dame e le pedine nere
        for r in 0..damiera.len() {
            for c in 0..damiera[r].len() {
                if damiera[r][c] == "n" {
                    pedine.push(vec![r, c]);
                } else if damiera[r][c] == "N" {
                    dame.push(vec![r, c]);
                }
            }
        }

        // Controllo se le pedine nere possono muovere o mangiare
        if pedine.len() > 0 {
            for n in 0..pedine.len() {
                row = pedine[n][0];
                col = pedine[n][1];

                // Controllo gli spostamenti
                if ((row as i32) + 1 <= 7 && (col as i32) + 1 <= 7)
                    && damiera[row + 1][col + 1] == " "
                {
                    // Spostamento DX giù
                    mossa = true;
                } else if ((row as i32) + 1 <= 7 && (col as i32) - 1 >= 0)
                    && damiera[row + 1][col - 1] == " "
                {
                    // Spostamento SX giù
                    mossa = true;
                }

                // Controllo le catture
                if ((row as i32) + 2 <= 7 && (col as i32) + 2 <= 7)
                    && damiera[row + 1][col + 1] == "b"
                    && damiera[row + 2][col + 2] == " "
                {
                    // Cattura DX giù
                    mossa = true;
                } else if ((row as i32) + 2 <= 7 && (col as i32) - 2 >= 0)
                    && damiera[row + 1][col - 1] == "b"
                    && damiera[row + 2][col - 2] == " "
                {
                    // Cattura verso SX giù
                    mossa = true;
                }
            }
        }

        // Controllo se le dame nere possono muovere o mangiare
        if dame.len() > 0 {
            for n in 0..dame.len() {
                row = dame[n][0];
                col = dame[n][1];

                // Controllo gli spostamenti
                if ((row as i32) - 1 >= 0 && (col as i32) - 1 >= 0)
                    && damiera[row - 1][col - 1] == " "
                {
                    // Spostamento SX su
                    mossa = true;
                } else if ((row as i32) - 1 >= 0 && (col as i32) + 1 <= 7)
                    && damiera[row - 1][col + 1] == " "
                {
                    // Spostamento DX su
                    mossa = true;
                }
                if ((row as i32) + 1 <= 7 && (col as i32) + 1 <= 7)
                    && damiera[row + 1][col + 1] == " "
                {
                    // Spostamento DX giù
                    mossa = true;
                } else if ((row as i32) + 1 <= 0 && (col as i32) - 1 >= 0)
                    && damiera[row + 1][col - 1] == " "
                {
                    // Spostamento SX giù
                    mossa = true;
                }

                // Controllo le catture
                if ((row as i32) - 2 >= 0 && (col as i32) - 2 >= 0)
                    && (damiera[row - 1][col - 1] == "b" || damiera[row - 1][col - 1] == "B")
                    && damiera[row - 2][col - 2] == " "
                {
                    // Catturo verso SX su
                    mossa = true;
                } else if ((row as i32) - 2 >= 0 && (col as i32) + 2 <= 7)
                    && (damiera[row - 1][col + 1] == "b" || damiera[row - 1][col + 1] == "B")
                    && damiera[row - 2][col + 2] == " "
                {
                    // Catturo verso DX su
                    mossa = true;
                } else if ((row as i32) + 2 <= 7 && (col as i32) + 2 <= 7)
                    && (damiera[row + 1][col + 1] == "b" || damiera[row + 1][col + 1] == "B")
                    && damiera[row + 2][col + 2] == " "
                {
                    // Catturo verso DX giù
                    mossa = true;
                } else if ((row as i32) + 2 <= 7 && (col as i32) - 2 >= 0)
                    && (damiera[row + 1][col - 1] == "b" || damiera[row + 1][col - 1] == "B")
                    && damiera[row + 1][col - 1] == " "
                {
                    // Catturo verso SX giù
                    mossa = true;
                }
            }
        }
    }

    // Restituisco se è possibile compiere una mossa per il prossimo turno
    mossa
}

pub(crate) async fn stampa_damiera(
    damiera: Vec<Vec<&str>>,
    giocatore: &mut Vec<Player>,
    spectators: &mut WriteHalf<DuplexStream>,
) {
    let mut stampa = String::new();

    for r in 0..damiera.len() {
        // Stampo la prima riga di lettere
        if r == 0 {
            _ = giocatore[0]
                .output
                .write(("\n   A  B  C  D  E  F  G  H\n").as_bytes())
                .await;
            _ = giocatore[1]
                .output
                .write(("\n   A  B  C  D  E  F  G  H\n").as_bytes())
                .await;
            _ = spectators
                .write(("\n   A  B  C  D  E  F  G  H\n").as_bytes())
                .await;
        }

        // Stampo i numeri a sx
        stampa.clear();
        stampa = (r + 1).to_string() + " ";
        _ = giocatore[0].output.write(stampa.as_bytes()).await;
        _ = giocatore[1].output.write(stampa.as_bytes()).await;
        _ = spectators.write(stampa.as_bytes()).await;

        for c in 0..damiera[r].len() {
            // Stampo la Damiera
            stampa.clear();
            stampa = "[".to_owned() + damiera[r][c] + "]";
            _ = giocatore[0].output.write(stampa.as_bytes()).await;
            _ = giocatore[1].output.write(stampa.as_bytes()).await;
            _ = spectators.write(stampa.as_bytes()).await;
        }

        // Stampo i numeri a dx
        stampa.clear();
        stampa = " ".to_owned() + &(r + 1).to_string() + "\n";
        _ = giocatore[0].output.write(stampa.as_bytes()).await;
        _ = giocatore[1].output.write(stampa.as_bytes()).await;
        _ = spectators.write(stampa.as_bytes()).await;

        // Stampo l'ultima riga di lettere
        if r == 7 {
            _ = giocatore[0]
                .output
                .write(("   A  B  C  D  E  F  G  H\n\n").as_bytes())
                .await;
            _ = giocatore[1]
                .output
                .write(("   A  B  C  D  E  F  G  H\n\n").as_bytes())
                .await;
            _ = spectators
                .write(("   A  B  C  D  E  F  G  H\n\n").as_bytes())
                .await;
        }
    }
}

pub(crate) async fn verifica_percorso_bianco(
    damiera: Vec<Vec<&str>>,
    giocatore: &mut Player,
    timer: Duration,
) -> Vec<String> {
    let mut err_mossa: bool = true;
    let mut mosse: Vec<String> = Vec::new();
    let mut stampa = String::new();

    while err_mossa {
        // Chiedo all'utente che mosse vuole fare
        mosse = percorso(giocatore, timer).await;

        // Verifico che il giocatore non abbia abbandonato
        if mosse[0] == "Err" {
            return mosse;
        }

        // Setto se sto muovendo una pedina o una dama
        let mut dama: bool = false;

        // Setto la posizione iniziale della pedina
        let mut pedina_r: usize = mosse[0]
            .chars()
            .nth(0)
            .expect("Error to 'pedina_r' in verifica_percorso_bianco")
            as usize
            - 0x30; // 0x30 = 0 nella tabella ASCII (Altrimenti non converte bene)
        let mut pedina_c: usize = mosse[0]
            .chars()
            .nth(1)
            .expect("Error to 'pedina_c' in verifica_percorso_bianco")
            as usize
            - 0x30; // 0x30 = 0 nella tabella ASCII (Altrimenti non converte bene)
        let mut mossa_r: usize;
        let mut mossa_c: usize;

        for (n, m) in mosse.iter().enumerate() {
            if n == 0 {
                // Controllo se ho selezionato la mia pedina
                if damiera[pedina_r][pedina_c] == "b" {
                    dama = false;
                } else if damiera[pedina_r][pedina_c] == "B" {
                    dama = true;
                } else {
                    _ = giocatore
                        .output
                        .write(
                            ("\nNon hai selezionato una tua pedina!\nRicorda che sei i bianchi.\n")
                                .as_bytes(),
                        )
                        .await;
                    break; // Esco dal for
                }
            } else {
                // Setto la mossa successiva
                mossa_r = m
                    .chars()
                    .nth(0)
                    .expect("Error to 'mossa_r' in verifica_percorso_bianco")
                    as usize
                    - 0x30; // 0x30 = 0 nella tabella ASCII (Altrimenti non converte bene)
                mossa_c = m
                    .chars()
                    .nth(1)
                    .expect("Error to 'mossa_c' in verifica_percorso_bianco")
                    as usize
                    - 0x30; // 0x30 = 0 nella tabella ASCII (Altrimenti non converte bene)

                // Verifico le mosse e le cattura
                if dama == false {
                    // Pedina

                    // Controllo se si fa una mossa o una cattura
                    if pedina_r - 1 == mossa_r && mosse.len() == 2 {
                        // Controllo se la mossa è in diagonale, di non uscire dalla damiera e se la casella di arrivo è vuota
                        if ((pedina_c == 0 && pedina_c + 1 == mossa_c)
                            || (pedina_c == 7 && pedina_c - 1 == mossa_c)
                            || ((pedina_c != 0 && pedina_c != 7)
                                && (pedina_c + 1 == mossa_c || pedina_c - 1 == mossa_c)))
                            && damiera[mossa_r][mossa_c] == " "
                        {
                            // Spiegazione controlli
                            // 1) mossa a DX se si è sulla colonna 0
                            // 2) mossa a SX se si è sulla colonna 7
                            // 3) mossa normale se si è al centro
                            err_mossa = false;
                        } else {
                            // La colonna è sbagliata
                            if damiera[mossa_r][mossa_c] != " " {
                                stampa.clear();
                                stampa = "\nSpostamento ".to_owned()
                                    + &stampa_mossa(pedina_r, pedina_c).await
                                    + " -> "
                                    + &stampa_mossa(mossa_r, mossa_c).await
                                    + " non valido!\nLa posizione scelta è gia occupata.\n";
                                _ = giocatore.output.write(stampa.as_bytes()).await;
                            } else {
                                stampa.clear();
                                stampa = "\nSpostamento ".to_owned() + &stampa_mossa(pedina_r, pedina_c).await + " -> " + &stampa_mossa(mossa_r, mossa_c).await +" non valido!\nNon ti stai spostando in diagonale o ti sei mosso di troppe colonne.\n";
                                _ = giocatore.output.write(stampa.as_bytes()).await;
                            }

                            break; // Esco dal for
                        }
                    } else if pedina_r - 2 == mossa_r {
                        // Controllo se la mossa è in diagonale, di non uscire dalla damiera e se la casella di arrivo è vuota
                        if (((pedina_c == 0 || pedina_c == 1)
                            && pedina_c + 2 == mossa_c
                            && damiera[pedina_r - 1][pedina_c + 1] == "n")
                            || ((pedina_c == 7 || pedina_c == 6)
                                && pedina_c - 2 == mossa_c
                                && damiera[pedina_r - 1][pedina_c - 1] == "n")
                            || ((pedina_c != 0 && pedina_c != 1 && pedina_c != 7 && pedina_c != 6)
                                && (pedina_c + 2 == mossa_c || pedina_c - 2 == mossa_c)
                                && ((mossa_c > pedina_c
                                    && damiera[pedina_r - 1][pedina_c + 1] == "n")
                                    || (mossa_c < pedina_c
                                        && damiera[pedina_r - 1][pedina_c - 1] == "n"))))
                            && damiera[mossa_r][mossa_c] == " "
                        {
                            // Spiegazione controlli
                            // 1) cattura a DX se si è sulla colonna 0 o 1
                            // 2) cattura a SX se si è sulla colonna 7 0 6
                            // 3) cattura normale se si è al centro

                            // La cattura è valida
                            err_mossa = false;
                        } else {
                            // La colonna è sbagliata
                            if damiera[mossa_r][mossa_c] != " " {
                                stampa.clear();
                                stampa = "\nCattura ".to_owned()
                                    + &stampa_mossa(pedina_r, pedina_c).await
                                    + " -> "
                                    + &stampa_mossa(mossa_r, mossa_c).await
                                    + " non valida!\nLa posizione scelta è gia occupata.\n";
                                _ = giocatore.output.write(stampa.as_bytes()).await;
                            } else {
                                stampa.clear();
                                stampa = "\nCattura ".to_owned() + &stampa_mossa(pedina_r, pedina_c).await + " -> " + &stampa_mossa(mossa_r, mossa_c).await +" non valida!\nNon ti stai spostando in diagonale o ti sei mosso di troppe colonne.\n";
                                _ = giocatore.output.write(stampa.as_bytes()).await;
                            }

                            err_mossa = true;
                            break; // Esco dal for
                        }
                    } else {
                        // La riga è sbagliata
                        if pedina_r == mossa_r {
                            stampa.clear();
                            stampa = "\nSpostamento ".to_owned()
                                + &stampa_mossa(pedina_r, pedina_c).await
                                + " -> "
                                + &stampa_mossa(mossa_r, mossa_c).await
                                + " non valido!\nPuoi muoverti solo in diagonale.\n";
                            _ = giocatore.output.write(stampa.as_bytes()).await;
                        } else {
                            stampa.clear();
                            stampa = "\nSpostamento ".to_owned() + &stampa_mossa(pedina_r, pedina_c).await + " -> " + &stampa_mossa(mossa_r, mossa_c).await +" non valido!\nLe righe sono troppo distanti o ti stai muovendo nel verso sbagliato.\n";
                            _ = giocatore.output.write(stampa.as_bytes()).await;
                        }

                        err_mossa = true;
                        break; // Esco dal for
                    }
                } else {
                    // Dama

                    // Controllo se la mossa è in diagonale, di non uscire dalla damiera e se si fa una mossa o una cattura
                    if ((pedina_r == 0 && pedina_r + 1 == mossa_r)
                        || (pedina_r == 7 && pedina_r - 1 == mossa_r)
                        || ((pedina_r != 0 && pedina_r != 7)
                            && (pedina_r - 1 == mossa_r || pedina_r + 1 == mossa_r)))
                        && mosse.len() == 2
                    {
                        // Spiegazione controlli
                        // 1) mossa GIU' se si è sulla riga 0
                        // 2) mossa SU se si è sulla riga 7
                        // 3) mossa normale se si è al centro

                        // Controllo se la mossa è in diagonale, di non uscire dalla damiera e se la casella di arrivo è vuota
                        if ((pedina_c == 0 && pedina_c + 1 == mossa_c)
                            || (pedina_c == 7 && pedina_c - 1 == mossa_c)
                            || ((pedina_c != 0 && pedina_c != 7)
                                && (pedina_c + 1 == mossa_c || pedina_c - 1 == mossa_c)))
                            && damiera[mossa_r][mossa_c] == " "
                        {
                            // Spiegazione controlli
                            // 1) mossa a DX se si è sulla colonna 0
                            // 2) mossa a SX se si è sulla colonna 7
                            // 3) mossa normale se si è al centro
                            err_mossa = false;
                        } else {
                            // La colonna è sbagliata
                            if damiera[mossa_r][mossa_c] != " " {
                                stampa.clear();
                                stampa = "\nSpostamento ".to_owned()
                                    + &stampa_mossa(pedina_r, pedina_c).await
                                    + " -> "
                                    + &stampa_mossa(mossa_r, mossa_c).await
                                    + " non valido!\nLa posizione scelta è gia occupata.\n";
                                _ = giocatore.output.write(stampa.as_bytes()).await;
                            } else {
                                stampa.clear();
                                stampa = "\nSpostamento ".to_owned() + &stampa_mossa(pedina_r, pedina_c).await + " -> " + &stampa_mossa(mossa_r, mossa_c).await +" non valido!\nNon ti stai spostando in diagonale o ti sei mosso di troppe colonne.\n";
                                _ = giocatore.output.write(stampa.as_bytes()).await;
                            }

                            break; // Esco dal for
                        }
                    } else if ((pedina_r == 0 || pedina_r == 1) && pedina_r + 2 == mossa_r)
                        || ((pedina_r == 7 || pedina_r == 6) && pedina_r - 2 == mossa_r)
                        || ((pedina_r != 0 && pedina_r != 1 && pedina_r != 7 && pedina_r != 6)
                            && (pedina_r - 2 == mossa_r || pedina_r + 2 == mossa_r))
                    {
                        // Spiegazione controlli
                        // 1) cattura GIU' se si è sulla riga 0 o 1
                        // 2) cattura SU se si è sulla riga 7 o 6
                        // 3) cattura normale se si è al centro

                        // Controllo se la mossa è in diagonale, di non uscire dalla damiera e se la casella di arrivo è vuota
                        if (((pedina_c == 0 || pedina_c == 1)
                            && pedina_c + 2 == mossa_c
                            && ((mossa_r > pedina_r
                                && damiera[pedina_r + 1][pedina_c + 1] == "n")
                                || (mossa_r > pedina_r
                                    && damiera[pedina_r + 1][pedina_c + 1] == "N")
                                || (mossa_r < pedina_r
                                    && damiera[pedina_r - 1][pedina_c + 1] == "n")
                                || (mossa_r < pedina_r
                                    && damiera[pedina_r - 1][pedina_c + 1] == "N")))
                            || ((pedina_c == 7 || pedina_c == 6)
                                && pedina_c - 2 == mossa_c
                                && ((mossa_r > pedina_r
                                    && damiera[pedina_r + 1][pedina_c - 1] == "n")
                                    || (mossa_r > pedina_r
                                        && damiera[pedina_r + 1][pedina_c - 1] == "N")
                                    || (mossa_r < pedina_r
                                        && damiera[pedina_r - 1][pedina_c - 1] == "n")
                                    || (mossa_r < pedina_r
                                        && damiera[pedina_r - 1][pedina_c - 1] == "N")))
                            || ((pedina_c != 0 && pedina_c != 1 && pedina_c != 7 && pedina_c != 6)
                                && (pedina_c + 2 == mossa_c || pedina_c - 2 == mossa_c)
                                && ((mossa_r > pedina_r
                                    && mossa_c > pedina_c
                                    && damiera[pedina_r + 1][pedina_c + 1] == "n")
                                    || (mossa_r > pedina_r
                                        && mossa_c > pedina_c
                                        && damiera[pedina_r + 1][pedina_c + 1] == "N")
                                    || (mossa_r < pedina_r
                                        && mossa_c > pedina_c
                                        && damiera[pedina_r - 1][pedina_c + 1] == "n")
                                    || (mossa_r < pedina_r
                                        && mossa_c > pedina_c
                                        && damiera[pedina_r - 1][pedina_c + 1] == "N")
                                    || (mossa_r > pedina_r
                                        && mossa_c < pedina_c
                                        && damiera[pedina_r + 1][pedina_c - 1] == "n")
                                    || (mossa_r > pedina_r
                                        && mossa_c < pedina_c
                                        && damiera[pedina_r + 1][pedina_c - 1] == "N")
                                    || (mossa_r < pedina_r
                                        && mossa_c < pedina_c
                                        && damiera[pedina_r - 1][pedina_c - 1] == "n")
                                    || (mossa_r < pedina_r
                                        && mossa_c < pedina_c
                                        && damiera[pedina_r - 1][pedina_c - 1] == "N"))))
                            && damiera[mossa_r][mossa_c] == " "
                        {
                            // Spiegazione controlli
                            // 1) cattura DX se si è sulla colonna 0 o 1
                            // 2) cattura SX se si è sulla colonna 7 o 6
                            // 3) cattura normale se si è al centro

                            // La cattura è valida
                            err_mossa = false;
                        } else {
                            // La colonna è sbagliata
                            if damiera[mossa_r][mossa_c] != " " {
                                stampa.clear();
                                stampa = "\nCattura ".to_owned()
                                    + &stampa_mossa(pedina_r, pedina_c).await
                                    + " -> "
                                    + &stampa_mossa(mossa_r, mossa_c).await
                                    + " non valida!\nLa posizione scelta è gia occupata.\n";
                                _ = giocatore.output.write(stampa.as_bytes()).await;
                            } else {
                                stampa.clear();
                                stampa = "\nCattura ".to_owned() + &stampa_mossa(pedina_r, pedina_c).await + " -> " + &stampa_mossa(mossa_r, mossa_c).await +" non valida!\nNon ti stai spostando in diagonale o ti sei mosso di troppe colonne.\n";
                                _ = giocatore.output.write(stampa.as_bytes()).await;
                            }

                            err_mossa = true;
                            break; // Esco dal for
                        }
                    } else {
                        // La riga è sbagliata
                        if pedina_r == mossa_r {
                            stampa.clear();
                            stampa = "\nSpostamento ".to_owned()
                                + &stampa_mossa(pedina_r, pedina_c).await
                                + " -> "
                                + &stampa_mossa(mossa_r, mossa_c).await
                                + " non valido!\nPuoi muoverti solo in diagonale.\n";
                            _ = giocatore.output.write(stampa.as_bytes()).await;
                        } else {
                            stampa.clear();
                            stampa = "\nSpostamento ".to_owned()
                                + &stampa_mossa(pedina_r, pedina_c).await
                                + " -> "
                                + &stampa_mossa(mossa_r, mossa_c).await
                                + " non valido!\nLe righe sono troppo distanti.\n";
                            _ = giocatore.output.write(stampa.as_bytes()).await;
                        }

                        err_mossa = true;
                        break; // Esco dal for
                    }
                }

                // Setto la nuova posizione della pedina
                pedina_r = mossa_r;
                pedina_c = mossa_c;
            }
        }
    }

    // Restituisco le mossa controllate
    mosse
}

pub(crate) async fn verifica_percorso_nero(
    damiera: Vec<Vec<&str>>,
    giocatore: &mut Player,
    timer: Duration,
) -> Vec<String> {
    let mut err_mossa: bool = true;
    let mut mosse: Vec<String> = Vec::new();
    let mut stampa: String = String::new();

    while err_mossa {
        // Chiedo all'utente che mosse vuole fare
        mosse = percorso(giocatore, timer).await;

        // Verifico che il giocatore non abbia abbandonato
        if mosse[0] == "Err" {
            return mosse;
        }

        // Setto se sto muovendo una pedina o una dama
        let mut dama: bool = false;

        // Setto la posizione iniziale della pedina
        let mut pedina_r: usize = mosse[0]
            .chars()
            .nth(0)
            .expect("Error to 'pedina_r' in verifica_percorso_nero")
            as usize
            - 0x30; // 0x30 = 0 nella tabella ASCII (Altrimenti non converte bene)
        let mut pedina_c: usize = mosse[0]
            .chars()
            .nth(1)
            .expect("Error to 'pedina_c' in verifica_percorso_nero")
            as usize
            - 0x30; // 0x30 = 0 nella tabella ASCII (Altrimenti non converte bene)
        let mut mossa_r: usize;
        let mut mossa_c: usize;

        for (n, m) in mosse.iter().enumerate() {
            if n == 0 {
                // Controllo se ho selezionato la mia pedina
                if damiera[pedina_r][pedina_c] == "n" {
                    dama = false;
                } else if damiera[pedina_r][pedina_c] == "N" {
                    dama = true;
                } else {
                    _ = giocatore
                        .output
                        .write(
                            ("\nNon hai selezionato una tua pedina!\nRicorda che sei i neri.\n")
                                .as_bytes(),
                        )
                        .await;
                    break; // Esco dal for
                }
            } else {
                // Setto la mossa successiva
                mossa_r = m
                    .chars()
                    .nth(0)
                    .expect("Error to 'mossa_r' in verifica_percorso_nero")
                    as usize
                    - 0x30; // 0x30 = 0 nella tabella ASCII (Altrimenti non converte bene)
                mossa_c = m
                    .chars()
                    .nth(1)
                    .expect("Error to 'mossa_c' in verifica_percorso_nero")
                    as usize
                    - 0x30; // 0x30 = 0 nella tabella ASCII (Altrimenti non converte bene)

                // Verifico le mosse e le cattura
                if dama == false {
                    // Pedina

                    // Controllo se si fa una mossa o una cattura
                    if pedina_r + 1 == mossa_r && mosse.len() == 2 {
                        // Controllo se la mossa è in diagonale, di non uscire dalla damiera e se la casella di arrivo è vuota
                        if ((pedina_c == 0 && pedina_c + 1 == mossa_c)
                            || (pedina_c == 7 && pedina_c - 1 == mossa_c)
                            || ((pedina_c != 0 && pedina_c != 7)
                                && (pedina_c + 1 == mossa_c || pedina_c - 1 == mossa_c)))
                            && damiera[mossa_r][mossa_c] == " "
                        {
                            // Spiegazione controlli
                            // 1) mossa a DX se si è sulla colonna 0
                            // 2) mossa a SX se si è sulla colonna 7
                            // 3) mossa normale se si è al centro
                            err_mossa = false;
                        } else {
                            // La colonna è sbagliata
                            if damiera[mossa_r][mossa_c] != " " {
                                stampa.clear();
                                stampa = "\nSpostamento ".to_owned()
                                    + &stampa_mossa(pedina_r, pedina_c).await
                                    + " -> "
                                    + &stampa_mossa(mossa_r, mossa_c).await
                                    + " non valido!\nLa posizione scelta è gia occupata.\n";
                                _ = giocatore.output.write(stampa.as_bytes()).await;
                            } else {
                                stampa.clear();
                                stampa = "\nSpostamento ".to_owned() + &stampa_mossa(pedina_r, pedina_c).await + " -> " + &stampa_mossa(mossa_r, mossa_c).await +" non valido!\nNon ti stai spostando in diagonale o ti sei mosso di troppe colonne.\n";
                                _ = giocatore.output.write(stampa.as_bytes()).await;
                            }

                            break; // Esco dal for
                        }
                    } else if pedina_r + 2 == mossa_r {
                        // Controllo se la mossa è in diagonale, di non uscire dalla damiera e se la casella di arrivo è vuota
                        if (((pedina_c == 0 || pedina_c == 1)
                            && pedina_c + 2 == mossa_c
                            && damiera[pedina_r + 1][pedina_c + 1] == "b")
                            || ((pedina_c == 7 || pedina_c == 6)
                                && pedina_c - 2 == mossa_c
                                && damiera[pedina_r + 1][pedina_c - 1] == "b")
                            || ((pedina_c != 0 && pedina_c != 1 && pedina_c != 7 && pedina_c != 6)
                                && (pedina_c + 2 == mossa_c || pedina_c - 2 == mossa_c)
                                && ((mossa_c > pedina_c
                                    && damiera[pedina_r + 1][pedina_c + 1] == "b")
                                    || (mossa_c < pedina_c
                                        && damiera[pedina_r + 1][pedina_c - 1] == "b"))))
                            && damiera[mossa_r][mossa_c] == " "
                        {
                            // Spiegazione controlli
                            // 1) cattura a DX se si è sulla colonna 0 o 1
                            // 2) cattura a SX se si è sulla colonna 7 0 6
                            // 3) cattura normale se si è al centro

                            // La cattura è valida
                            err_mossa = false;
                        } else {
                            // La colonna è sbagliata
                            if damiera[mossa_r][mossa_c] != " " {
                                stampa.clear();
                                stampa = "\nCattura ".to_owned()
                                    + &stampa_mossa(pedina_r, pedina_c).await
                                    + " -> "
                                    + &stampa_mossa(mossa_r, mossa_c).await
                                    + " non valida!\nLa posizione scelta è gia occupata.\n";
                                _ = giocatore.output.write(stampa.as_bytes()).await;
                            } else {
                                stampa.clear();
                                stampa = "\nCattura ".to_owned() + &stampa_mossa(pedina_r, pedina_c).await + " -> " + &stampa_mossa(mossa_r, mossa_c).await +" non valida!\nNon ti stai spostando in diagonale o ti sei mosso di troppe colonne.\n";
                                _ = giocatore.output.write(stampa.as_bytes()).await;
                            }

                            err_mossa = true;
                            break; // Esco dal for
                        }
                    } else {
                        // La riga è sbagliata
                        if pedina_r == mossa_r {
                            stampa.clear();
                            stampa = "\nSpostamento ".to_owned()
                                + &stampa_mossa(pedina_r, pedina_c).await
                                + " -> "
                                + &stampa_mossa(mossa_r, mossa_c).await
                                + " non valido!\nPuoi muoverti solo in diagonale.\n";
                            _ = giocatore.output.write(stampa.as_bytes()).await;
                        } else {
                            stampa.clear();
                            stampa = "\nSpostamento ".to_owned() + &stampa_mossa(pedina_r, pedina_c).await + " -> " + &stampa_mossa(mossa_r, mossa_c).await +" non valido!\nLe righe sono troppo distanti o ti stai muovendo nel verso sbagliato.\n";
                            _ = giocatore.output.write(stampa.as_bytes()).await;
                        }

                        err_mossa = true;
                        break; // Esco dal for
                    }
                } else {
                    // Dama

                    // Controllo se la mossa è in diagonale, di non uscire dalla damiera e se si fa una mossa o una cattura
                    if ((pedina_r == 0 && pedina_r + 1 == mossa_r)
                        || (pedina_r == 7 && pedina_r - 1 == mossa_r)
                        || ((pedina_r != 0 && pedina_r != 7)
                            && (pedina_r - 1 == mossa_r || pedina_r + 1 == mossa_r)))
                        && mosse.len() == 2
                    {
                        // Spiegazione controlli
                        // 1) mossa GIU' se si è sulla riga 0
                        // 2) mossa SU se si è sulla riga 7
                        // 3) mossa normale se si è al centro

                        // Controllo se la mossa è in diagonale, di non uscire dalla damiera e se la casella di arrivo è vuota
                        if ((pedina_c == 0 && pedina_c + 1 == mossa_c)
                            || (pedina_c == 7 && pedina_c - 1 == mossa_c)
                            || ((pedina_c != 0 && pedina_c != 7)
                                && (pedina_c + 1 == mossa_c || pedina_c - 1 == mossa_c)))
                            && damiera[mossa_r][mossa_c] == " "
                        {
                            // Spiegazione controlli
                            // 1) mossa a DX se si è sulla colonna 0
                            // 2) mossa a SX se si è sulla colonna 7
                            // 3) mossa normale se si è al centro
                            err_mossa = false;
                        } else {
                            // La colonna è sbagliata
                            if damiera[mossa_r][mossa_c] != " " {
                                stampa.clear();
                                stampa = "\nSpostamento ".to_owned()
                                    + &stampa_mossa(pedina_r, pedina_c).await
                                    + " -> "
                                    + &stampa_mossa(mossa_r, mossa_c).await
                                    + " non valido!\nLa posizione scelta è gia occupata.\n";
                                _ = giocatore.output.write(stampa.as_bytes()).await;
                            } else {
                                stampa.clear();
                                stampa = "\nSpostamento ".to_owned() + &stampa_mossa(pedina_r, pedina_c).await + " -> " + &stampa_mossa(mossa_r, mossa_c).await +" non valido!\nNon ti stai spostando in diagonale o ti sei mosso di troppe colonne.\n";
                                _ = giocatore.output.write(stampa.as_bytes()).await;
                            }

                            break; // Esco dal for
                        }
                    } else if ((pedina_r == 0 || pedina_r == 1) && pedina_r + 2 == mossa_r)
                        || ((pedina_r == 7 || pedina_r == 6) && pedina_r - 2 == mossa_r)
                        || ((pedina_r != 0 && pedina_r != 1 && pedina_r != 7 && pedina_r != 6)
                            && (pedina_r - 2 == mossa_r || pedina_r + 2 == mossa_r))
                    {
                        // Spiegazione controlli
                        // 1) cattura GIU' se si è sulla riga 0 o 1
                        // 2) cattura SU se si è sulla riga 7 o 6
                        // 3) cattura normale se si è al centro

                        // Controllo se la mossa è in diagonale, di non uscire dalla damiera e se la casella di arrivo è vuota
                        if (((pedina_c == 0 || pedina_c == 1)
                            && pedina_c + 2 == mossa_c
                            && ((mossa_r > pedina_r
                                && damiera[pedina_r + 1][pedina_c + 1] == "b")
                                || (mossa_r > pedina_r
                                    && damiera[pedina_r + 1][pedina_c + 1] == "B")
                                || (mossa_r < pedina_r
                                    && damiera[pedina_r - 1][pedina_c + 1] == "b")
                                || (mossa_r < pedina_r
                                    && damiera[pedina_r - 1][pedina_c + 1] == "B")))
                            || ((pedina_c == 7 || pedina_c == 6)
                                && pedina_c - 2 == mossa_c
                                && ((mossa_r > pedina_r
                                    && damiera[pedina_r + 1][pedina_c - 1] == "b")
                                    || (mossa_r > pedina_r
                                        && damiera[pedina_r + 1][pedina_c - 1] == "B")
                                    || (mossa_r < pedina_r
                                        && damiera[pedina_r - 1][pedina_c - 1] == "b")
                                    || (mossa_r < pedina_r
                                        && damiera[pedina_r - 1][pedina_c - 1] == "B")))
                            || ((pedina_c != 0 && pedina_c != 1 && pedina_c != 7 && pedina_c != 6)
                                && (pedina_c + 2 == mossa_c || pedina_c - 2 == mossa_c)
                                && ((mossa_r > pedina_r
                                    && mossa_c > pedina_c
                                    && damiera[pedina_r + 1][pedina_c + 1] == "b")
                                    || (mossa_r > pedina_r
                                        && mossa_c > pedina_c
                                        && damiera[pedina_r + 1][pedina_c + 1] == "B")
                                    || (mossa_r < pedina_r
                                        && mossa_c > pedina_c
                                        && damiera[pedina_r - 1][pedina_c + 1] == "b")
                                    || (mossa_r < pedina_r
                                        && mossa_c > pedina_c
                                        && damiera[pedina_r - 1][pedina_c + 1] == "B")
                                    || (mossa_r > pedina_r
                                        && mossa_c < pedina_c
                                        && damiera[pedina_r + 1][pedina_c - 1] == "b")
                                    || (mossa_r > pedina_r
                                        && mossa_c < pedina_c
                                        && damiera[pedina_r + 1][pedina_c - 1] == "B")
                                    || (mossa_r < pedina_r
                                        && mossa_c < pedina_c
                                        && damiera[pedina_r - 1][pedina_c - 1] == "b")
                                    || (mossa_r < pedina_r
                                        && mossa_c < pedina_c
                                        && damiera[pedina_r - 1][pedina_c - 1] == "B"))))
                            && damiera[mossa_r][mossa_c] == " "
                        {
                            // Spiegazione controlli
                            // 1) cattura DX se si è sulla colonna 0 o 1
                            // 2) cattura SX se si è sulla colonna 7 o 6
                            // 3) cattura normale se si è al centro

                            // La cattura è valida
                            err_mossa = false;
                        } else {
                            // La colonna è sbagliata
                            if damiera[mossa_r][mossa_c] != " " {
                                stampa.clear();
                                stampa = "\nCattura ".to_owned()
                                    + &stampa_mossa(pedina_r, pedina_c).await
                                    + " -> "
                                    + &stampa_mossa(mossa_r, mossa_c).await
                                    + " non valida!\nLa posizione scelta è gia occupata.\n";
                                _ = giocatore.output.write(stampa.as_bytes()).await;
                            } else {
                                stampa.clear();
                                stampa = "\nCattura ".to_owned() + &stampa_mossa(pedina_r, pedina_c).await + " -> " + &stampa_mossa(mossa_r, mossa_c).await +" non valida!\nNon ti stai spostando in diagonale o ti sei mosso di troppe colonne.\n";
                                _ = giocatore.output.write(stampa.as_bytes()).await;
                            }

                            err_mossa = true;
                            break; // Esco dal for
                        }
                    } else {
                        // La riga è sbagliata
                        if pedina_r == mossa_r {
                            stampa.clear();
                            stampa = "\nSpostamento ".to_owned()
                                + &stampa_mossa(pedina_r, pedina_c).await
                                + " -> "
                                + &stampa_mossa(mossa_r, mossa_c).await
                                + " non valido!\nPuoi muoverti solo in diagonale.\n";
                            _ = giocatore.output.write(stampa.as_bytes()).await;
                        } else {
                            stampa.clear();
                            stampa = "\nSpostamento ".to_owned()
                                + &stampa_mossa(pedina_r, pedina_c).await
                                + " -> "
                                + &stampa_mossa(mossa_r, mossa_c).await
                                + " non valido!\nLe righe sono troppo distanti.\n";
                            _ = giocatore.output.write(stampa.as_bytes()).await;
                        }

                        err_mossa = true;
                        break; // Esco dal for
                    }
                }

                // Setto la nuova posizione della pedina
                pedina_r = mossa_r;
                pedina_c = mossa_c;
            }
        }
    }

    // Restituisco le mossa controllate
    mosse
}

pub(crate) async fn aggionra_damiera<'a>(
    percorso_valido: Vec<String>,
    mut damiera: Vec<Vec<&'a str>>,
) -> Vec<Vec<&'a str>> {
    // Setto la posizione iniziale della pedina
    let mut pedina_r: usize = percorso_valido[0]
        .chars()
        .nth(0)
        .expect("Error to 'pedina_r' in aggionra_damiera") as usize
        - 0x30; // 0x30 = 0 nella tabella ASCII (Altrimenti non converte bene)
    let mut pedina_c: usize = percorso_valido[0]
        .chars()
        .nth(1)
        .expect("Error to 'pedina_c' in aggionra_damiera") as usize
        - 0x30; // 0x30 = 0 nella tabella ASCII (Altrimenti non converte bene);
    let mut mossa_r: usize;
    let mut mossa_c: usize;

    // Identifico la pedina che devo muovere
    let pedina: &str = damiera[pedina_r][pedina_c];

    // Inizio l'aggiornamento della damiera
    for (n, m) in percorso_valido.iter().enumerate() {
        if n == 0 {
            // Cancello la posizione iniziale della pedina
            damiera[pedina_r][pedina_c] = " ";
        } else {
            // Setto la mossa successiva
            mossa_r = m
                .chars()
                .nth(0)
                .expect("Error to 'mossa_r' in aggionra_damiera") as usize
                - 0x30; // 0x30 = 0 nella tabella ASCII (Altrimenti non converte bene)
            mossa_c = m
                .chars()
                .nth(1)
                .expect("Error to 'mossa_c' in aggionra_damiera") as usize
                - 0x30; // 0x30 = 0 nella tabella ASCII (Altrimenti non converte bene)

            // Controllo se si fa una mossa o una cattura
            if (pedina_r == 0 && pedina_r + 1 == mossa_r)
                || (pedina_r == 7 && pedina_r - 1 == mossa_r)
                || ((pedina_r != 0 && pedina_r != 7)
                    && (pedina_r - 1 == mossa_r || pedina_r + 1 == mossa_r))
            {
                // Setto la nuova posizione della pedina
                // e controllo se ho fatto dama
                damiera[mossa_r][mossa_c] = dama(pedina, mossa_r).await;
            } else {
                // Cancello le pedine mangiate
                if mossa_r < pedina_r && mossa_c < pedina_c {
                    damiera[pedina_r - 1][pedina_c - 1] = " ";
                } else if mossa_r < pedina_r && mossa_c > pedina_c {
                    damiera[pedina_r - 1][pedina_c + 1] = " ";
                } else if mossa_r > pedina_r && mossa_c > pedina_c {
                    damiera[pedina_r + 1][pedina_c + 1] = " ";
                } else if mossa_r > pedina_r && mossa_c < pedina_c {
                    damiera[pedina_r + 1][pedina_c - 1] = " ";
                } else {
                    println!("Errore: Qualcosa è andato storto nell'aggiornamento della damiera!");
                }

                // Se mi trovo all'ultima mossa setto la nuova posizione della pedina
                // e controllo se ho fatto dama
                if n + 1 == percorso_valido.len() {
                    damiera[mossa_r][mossa_c] = dama(pedina, mossa_r).await;
                }
            }

            // Setto la nuova posizione della pedina
            pedina_r = mossa_r;
            pedina_c = mossa_c;
        }
    }

    // Restituisco la damiera aggiornata
    damiera
}

pub(crate) async fn dama(mut pedina: &str, mossa_r: usize) -> &str {
    // Verifico se ho fatto dama
    if pedina == "b" && mossa_r == 0 {
        pedina = "B";
        pedina
    } else if pedina == "n" && mossa_r == 7 {
        pedina = "N";
        pedina
    } else {
        pedina
    }
}

async fn percorso(giocatore: &mut Player, timer: Duration) -> Vec<String> {
    // Inizializzo le variabili
    let mut percorso: String = String::new();
    let mut mosse: Vec<String> = Vec::new();
    let mut err_mosse: bool = true;

    // Controllo le mosse siano all'interno della damiera
    while err_mosse {
        _ = giocatore.output.write(("Inserisci la pedina che vui muovere e poi le mosse che vuio fare\nEs > 6A 5B oppure 6A 4C 2A oppure 6A 4C 2A ...\n").as_bytes()).await;

        // Pulisco la variabili altrimenti si portano dietro tutti i valori precedenti
        percorso.clear();
        mosse.clear();

        // Controllo che il giocatore non abbia abbandonato la partita
        match timeout(timer, giocatore.input.read_line(&mut percorso)).await {
            Err(_) | Ok(Err(_)) => {
                _ = giocatore
                    .output
                    .write(
                        ("\nTempo scaduto, hai esaurito il tempo per fare una mossa!\n").as_bytes(),
                    )
                    .await;
                mosse.insert(0, "Err".to_string());
                return mosse;
            }
            Ok(Ok(x)) => {
                // Se viene ritornato 0 vuol dire che il giocatore ha abbandonato
                if x == 0 {
                    mosse.insert(0, "Err".to_string());
                    return mosse;
                }
            }
        };

        // Elimino tutti gli elementi non necessari dalla stringa
        percorso = percorso.replace(&['\n', '\r', '\t'][..], "");
        // Inserisco gli elementi in un vettore
        mosse = percorso.split(" ").map(|x| x.into()).collect();
        // Elimino lo spazio finale dal vettore se presente
        mosse.retain(|x| x != "");

        for i in 0..mosse.len() {
            let m = &mosse[i];

            // Verifico che ogni mossa abbia 2 caratteri
            if m.len() != 2 {
                break;
            }

            let pos = conv_mossa_in_num(&m).await;

            // Verifico che i valori inseriti siano all'interno della damiera
            if pos.len() != 2 {
                break;
            }

            // Cavo l'elemento dato dall'utente e lo rimpiazzo con quello convertito
            mosse.remove(i);
            mosse.insert(i, pos);

            // Controllo se tutte le mosse sono state convertite
            if i + 1 == mosse.len() && mosse.len() > 1 {
                err_mosse = false;
            }
        }

        if err_mosse {
            _ = giocatore
                .output
                .write(("\nMossa non valida riprova!\n").as_bytes())
                .await;
        }
    }

    // Restituisco le mosse convertire
    mosse
}

pub(crate) async fn conv_mossa_in_num(posizione: &str) -> String {
    // Convertitore di riga
    let row: &str = match posizione.chars().nth(0) {
        Some('1') => "0",
        Some('2') => "1",
        Some('3') => "2",
        Some('4') => "3",
        Some('5') => "4",
        Some('6') => "5",
        Some('7') => "6",
        Some('8') => "7",
        _ => "Null",
    };

    // Convertitore di colonna
    let col: &str = match posizione.chars().nth(1) {
        Some('a') | Some('A') => "0",
        Some('b') | Some('B') => "1",
        Some('c') | Some('C') => "2",
        Some('d') | Some('D') => "3",
        Some('e') | Some('E') => "4",
        Some('f') | Some('F') => "5",
        Some('g') | Some('G') => "6",
        Some('h') | Some('H') => "7",
        _ => "Null",
    };

    // Unisco le due stringhe convertite e le restituisco
    row.to_owned() + col
}

pub(crate) async fn conv_mossa_in_alfanum(posizione: &str) -> String {
    // Convertitore di riga
    let row: &str = match posizione.chars().nth(0) {
        Some('0') => "1",
        Some('1') => "2",
        Some('2') => "3",
        Some('3') => "4",
        Some('4') => "5",
        Some('5') => "6",
        Some('6') => "7",
        Some('7') => "8",
        _ => "Null",
    };

    // Convertitore di colonna
    let col: &str = match posizione.chars().nth(1) {
        Some('0') => "A",
        Some('1') => "B",
        Some('2') => "C",
        Some('3') => "D",
        Some('4') => "E",
        Some('5') => "F",
        Some('6') => "G",
        Some('7') => "H",
        _ => "Null",
    };

    // Unisco le due stringhe convertite e le restituisco
    row.to_owned() + col
}

pub(crate) async fn stampa_mossa(row: usize, col: usize) -> String {
    // Convertitore di riga
    let r: &str = match row {
        0 => "1",
        1 => "2",
        2 => "3",
        3 => "4",
        4 => "5",
        5 => "6",
        6 => "7",
        7 => "8",
        _ => "Null",
    };

    // Convertitore di colonna
    let c: &str = match col {
        0 => "A",
        1 => "B",
        2 => "C",
        3 => "D",
        4 => "E",
        5 => "F",
        6 => "G",
        7 => "H",
        _ => "Null",
    };

    // Unisco le due stringhe convertite e le restituisco
    r.to_owned() + c
}
