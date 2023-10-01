use super::logic;
use crate::game;
use async_trait::async_trait;
use tracing::error;
use rand::{Rng, SeedableRng};
use rand::rngs::SmallRng;
use tokio::io::{split, AsyncBufReadExt, AsyncWriteExt, BufReader, DuplexStream};

#[derive(Debug)]
pub(crate) struct Bot {}

#[async_trait]
impl game::Bot for Bot {
    async fn start(&mut self, stream: DuplexStream) {
        let (input, mut output) = split(stream);
        let mut input = BufReader::new(input);
        lnin!(input); // Leggo il mio nome
        lnin!(input); // Leggo il nome dell'avversario

        let mut fine_partita: bool = false;

        let mut damiera: Vec<Vec<&str>> = vec![vec![" ", "n", " ", "n", " ", "n", " ", "n"],
                                               vec!["n", " ", "n", " ", "n", " ", "n", " "],
                                               vec![" ", "n", " ", "n", " ", "n", " ", "n"],
                                               vec![" ", " ", " ", " ", " ", " ", " ", " "],
                                               vec![" ", " ", " ", " ", " ", " ", " ", " "],
                                               vec!["b", " ", "b", " ", "b", " ", "b", " "],
                                               vec![" ", "b", " ", "b", " ", "b", " ", "b"],
                                               vec!["b", " ", "b", " ", "b", " ", "b", " "]
                                              ];

        let mut mossa_scelta: String;

        while !fine_partita {
        
            match lnin!(input).as_str() {
                "Bianchi" => {
                    // Converto la damiera
                    damiera = logic::converti_damiera(damiera.clone(), lnin!(input).to_string()).await;

                    // Faccio muovere le pedine binche al bot
                    (mossa_scelta, damiera) = bot_bianco(damiera.clone()).await;
                    
                    // Inio la mossa scelta e la damiera aggiornata
                    lnout!(output, mossa_scelta.clone() + "|" + &damiera.clone().into_iter()
                                                  .map(|c| c.into_iter().map(|p| match p { 
                                                                            " " => "_",
                                                                            _ => p
                                                                        })
                                                                        .collect::<Vec<&str>>()
                                                                        .join(""))
                                                  .collect::<Vec<String>>()
                                                  .join(""));
                },
                "Neri" => {
                    // Converto la damiera
                    damiera = logic::converti_damiera(damiera.clone(), lnin!(input).to_string()).await;

                    // Faccio muovere le pedine binche al bot
                    (mossa_scelta, damiera) = bot_nero(damiera.clone()).await;
                    
                    // Inio la mossa scelta e la damiera aggiornata
                    lnout!(output, mossa_scelta.clone() + "|" + &damiera.clone().into_iter()
                                                  .map(|c| c.into_iter().map(|p| match p { 
                                                                            " " => "_",
                                                                            _ => p
                                                                        })
                                                                        .collect::<Vec<&str>>()
                                                                        .join(""))
                                                  .collect::<Vec<String>>()
                                                  .join(""));
                },
                "Stop" => fine_partita = true,
                _ => continue, // Attendo il mio turno
            };
        }
    }
}

pub(crate) async fn bot_bianco<'a>(mut damiera: Vec<Vec<&'a str>>) -> (String, Vec<Vec<&'a str>>){
    let mut dame = Vec::new();
    let mut pedine = Vec::new();
    let mut mossa_scelta:String = String::new();

    // Prelevo le Dame e le pedine
    for r in 0..damiera.len() {
        for c in 0..damiera[r].len() {
        
            if damiera[r][c] == "b" {
                pedine.push(vec![r, c]);
            }
            else if damiera[r][c] == "B" {
                dame.push(vec![r, c]);
            }
        }
    }

    let mut row: usize;
    let mut col: usize;
    let mut cattura = false;

    // Mangio con la prima dama disponibile
    if dame.len() != 0 {
        
        for n in 0..dame.len() {
            
            row = dame[n][0];
            col = dame[n][1];

            if ((row as i32) - 2 >= 0 && (col as i32) - 2 >= 0) && 
               (damiera[row - 1][col - 1] == "n" || damiera[row - 1][col - 1] == "N") &&
               damiera[row-2][col-2] == " " {
                
                damiera[row][col] = " "; // Cancello la posizione iniziale
                damiera[row-1][col-1] = " "; // Cancello la pedina avversaria mangiata
                damiera[row-2][col-2] = "B"; // Setto la nuova posizione della pedina
                
                mossa_scelta = logic::stampa_mossa(row, col).await + " " + &logic::stampa_mossa(row-2, col-2).await;

                cattura = true;
                break; // Ho mangiato ed esco dal for
            }
            else if ((row as i32) - 2 >= 0 && (col as i32) + 2 <= 7) && 
                    (damiera[row - 1][col + 1] == "n" || damiera[row - 1][col + 1] == "N") &&
                    damiera[row-2][col+2] == " "{
                
                damiera[row][col] = " "; // Cancello la posizione iniziale
                damiera[row-1][col+1] = " "; // Cancello la pedina avversaria mangiata
                damiera[row-2][col+2] = "B"; // Setto la nuova posizione della pedina

                mossa_scelta = logic::stampa_mossa(row, col).await + " " + &logic::stampa_mossa(row-2, col+2).await;

                cattura = true;
                break; // Ho mangiato ed esco dal for
            }
            else if ((row as i32) + 2 <= 7 && (col as i32) + 2 <= 7) && 
                    (damiera[row + 1][col + 1] == "n" || damiera[row + 1][col + 1] == "N") &&
                    damiera[row+2][col+2] == " "{
                
                damiera[row][col] = " "; // Cancello la posizione iniziale
                damiera[row+1][col+1] = " "; // Cancello la pedina avversaria mangiata
                damiera[row+2][col+2] = "B"; // Setto la nuova posizione della pedina

                mossa_scelta = logic::stampa_mossa(row, col).await + " " + &logic::stampa_mossa(row+2, col+2).await;

                cattura = true;
                break; // Ho mangiato ed esco dal for
            }
            else if ((row as i32) + 2 <= 7 && (col as i32) - 2 >= 0) && 
                    (damiera[row + 1][col - 1] == "n" || damiera[row + 1][col - 1] == "N") &&
                    damiera[row+2][col-2] == " "{
                
                damiera[row][col] = " "; // Cancello la posizione iniziale
                damiera[row+1][col-1] = " "; // Cancello la pedina avversaria mangiata
                damiera[row+2][col-2] = "B"; // Setto la nuova posizione della pedina

                mossa_scelta = logic::stampa_mossa(row, col).await + " " + &logic::stampa_mossa(row+2, col-2).await;

                cattura = true;
                break; // Ho mangiato ed esco dal for
            }
        }
    }

    // Mangio con la prima pedina disponibile
    if pedine.len() != 0 && cattura == false{

        for n in 0..pedine.len() {
            
            row = pedine[n][0];
            col = pedine[n][1];

            if ((row as i32) - 2 >= 0 && (col as i32) - 2 >= 0) && 
               damiera[row - 1][col - 1] == "n" &&
               damiera[row-2][col-2] == " "{
                
                damiera[row][col] = " "; // Cancello la posizione iniziale
                damiera[row-1][col-1] = " "; // Cancello la pedina avversaria mangiata
                damiera[row-2][col-2] = logic::dama("b", row-2).await; // Setto la nuova posizione della pedina e controllo se ho fatto dama

                mossa_scelta = logic::stampa_mossa(row, col).await + " " + &logic::stampa_mossa(row-2, col-2).await;

                cattura = true;
                break; // Ho mangiato ed esco dal for
            }
            else if ((row as i32) - 2 >= 0 && (col as i32) + 2 <= 7) && 
                    damiera[row - 1][col + 1] == "n" &&
                    damiera[row-2][col+2] == " "{
                
                damiera[row][col] = " "; // Cancello la posizione iniziale
                damiera[row-1][col+1] = " "; // Cancello la pedina avversaria mangiata
                damiera[row-2][col+2] = logic::dama("b", row-2).await; // Setto la nuova posizione della pedina e controllo se ho fatto dama

                mossa_scelta = logic::stampa_mossa(row, col).await + " " + &logic::stampa_mossa(row-2, col+2).await;

                cattura = true;
                break; // Ho mangiato ed esco dal for
            }
        }
    }

    // Se non ho mangiato faccio una mossa in maniera casuale
    if cattura == false{
        let n_pedine = pedine.len() + dame.len();
        let mut rng = SmallRng::from_entropy();
        let mut scelta = rng.gen_range(0..n_pedine);
        let mut mossa = false;
        let mut continua = true;

        if scelta < dame.len(){

            while continua == true || scelta != dame.len() {
                
                row = dame[scelta][0];
                col = dame[scelta][1];

                if ((row as i32) - 1 >= 0 && (col as i32) - 1 >= 0) &&
                   damiera[row-1][col-1] == " " {
                    
                    damiera[row][col] = " "; // Cancello la posizione iniziale
                    damiera[row-1][col-1] = "B"; // Setto la nuova posizione della pedina

                    mossa_scelta = logic::stampa_mossa(row, col).await + " " + &logic::stampa_mossa(row-1, col-1).await;

                    mossa = true;
                    break; // Ho fatto una mossa ed esco dal for
                }
                else if ((row as i32) - 1 >= 0 && (col as i32) + 1 <= 7) &&
                        damiera[row-1][col+1] == " "{
                    
                    damiera[row][col] = " "; // Cancello la posizione iniziale
                    damiera[row-1][col+1] = "B"; // Setto la nuova posizione della pedina

                    mossa_scelta = logic::stampa_mossa(row, col).await + " " + &logic::stampa_mossa(row-1, col+1).await;

                    mossa = true;
                    break; // Ho fatto una mossa ed esco dal for
                }
                else if ((row as i32) + 1 <= 7 && (col as i32) + 1 <= 7) &&
                        damiera[row+1][col+1] == " "{
                    
                    damiera[row][col] = " "; // Cancello la posizione iniziale
                    damiera[row+1][col+1] = "B"; // Setto la nuova posizione della pedina

                    mossa_scelta = logic::stampa_mossa(row, col).await + " " + &logic::stampa_mossa(row+1, col+1).await;

                    mossa = true;
                    break; // Ho fatto una mossa ed esco dal for
                }
                else if ((row as i32) + 1 <= 7 && (col as i32) - 1 >= 0) &&
                        damiera[row+1][col-1] == " "{
                    
                    damiera[row][col] = " "; // Cancello la posizione iniziale
                    damiera[row+1][col-1] = "B"; // Setto la nuova posizione della pedina

                    mossa_scelta = logic::stampa_mossa(row, col).await + " " + &logic::stampa_mossa(row+1, col-1).await;

                    mossa = true;
                    break; // Ho fatto una mossa ed esco dal for
                }
                else {

                    if scelta == dame.len() - 1 {
                        continua = false;
                        scelta = 0;
                    }
                    else {
                        scelta += 1;
                    }
                }
            }
        }

        if mossa == false {
            // Reimposto la mossa su false
            continua = true;
            // Resetto la scelta considerando solo le pedine
            scelta = rng.gen_range(0..pedine.len());

            while continua == true || scelta != pedine.len() {
                
                row = pedine[scelta][0];
                col = pedine[scelta][1];

                if ((row as i32) - 1 >= 0 && (col as i32) - 1 >= 0) &&
                   damiera[row-1][col-1] == " "{

                    damiera[row][col] = " "; // Cancello la posizione iniziale
                    damiera[row-1][col-1] = logic::dama("b", row-1).await; // Setto la nuova posizione della pedina e controllo se ho fatto dama

                    mossa_scelta = logic::stampa_mossa(row, col).await + " " + &logic::stampa_mossa(row-1, col-1).await;
                    
                    break; // Ho fatto una mossa ed esco dal for
                }
                else if ((row as i32) - 1 >= 0 && (col as i32) + 1 <= 7) &&
                        damiera[row-1][col+1] == " "{

                    damiera[row][col] = " "; // Cancello la posizione iniziale
                    damiera[row-1][col+1] = logic::dama("b", row-1).await; // Setto la nuova posizione della pedina e controllo se ho fatto dama

                    mossa_scelta = logic::stampa_mossa(row, col).await + " " + &logic::stampa_mossa(row-1, col+1).await;

                    break; // Ho fatto una mossa ed esco dal for
                }
                else {

                    if scelta == pedine.len() - 1 {
                        continua = false;
                        scelta = 0;
                    }else {
                        scelta += 1;    
                    }
                }
            }
        }
    }

    // Ritorno la damiera sia che ho eseguito una mossa che non
    (mossa_scelta, damiera)
}

pub(crate) async fn bot_nero<'a>(mut damiera: Vec<Vec<&'a str>>) -> (String, Vec<Vec<&'a str>>){
    let mut dame = Vec::new();
    let mut pedine = Vec::new();
    let mut mossa_scelta:String = String::new();

    // Prelevo le Dame e le pedine
    for r in 0..damiera.len() {
        for c in 0..damiera[r].len() {
        
            if damiera[r][c] == "n" {
                pedine.push(vec![r, c]);
            }
            else if damiera[r][c] == "N" {
                dame.push(vec![r, c]);
            }
        }
    }

    let mut row: usize;
    let mut col: usize;
    let mut cattura = false;

    // Mangio con la prima dama disponibile
    if dame.len() != 0 {
        
        for n in 0..dame.len() {
            
            row = dame[n][0];
            col = dame[n][1];

            if ((row as i32) - 2 >= 0 && (col as i32) - 2 >= 0) && 
               (damiera[row - 1][col - 1] == "b" || damiera[row - 1][col - 1] == "B") &&
               damiera[row-2][col-2] == " " {
                
                damiera[row][col] = " "; // Cancello la posizione iniziale
                damiera[row-1][col-1] = " "; // Cancello la pedina avversaria mangiata
                damiera[row-2][col-2] = "N"; // Setto la nuova posizione della pedina

                mossa_scelta = logic::stampa_mossa(row, col).await + " " + &logic::stampa_mossa(row-2, col-2).await;

                cattura = true;
                break; // Ho mangiato ed esco dal for
            }
            else if ((row as i32) - 2 >= 0 && (col as i32) + 2 <= 7) && 
                    (damiera[row - 1][col + 1] == "b" || damiera[row - 1][col + 1] == "B") &&
                    damiera[row-2][col+2] == " "{
                
                damiera[row][col] = " "; // Cancello la posizione iniziale
                damiera[row-1][col+1] = " "; // Cancello la pedina avversaria mangiata
                damiera[row-2][col+2] = "N"; // Setto la nuova posizione della pedina

                mossa_scelta = logic::stampa_mossa(row, col).await + " " + &logic::stampa_mossa(row-2, col+2).await;

                cattura = true;
                break; // Ho mangiato ed esco dal for
            }
            else if ((row as i32) + 2 <= 7 && (col as i32) + 2 <= 7) && 
                    (damiera[row + 1][col + 1] == "b" || damiera[row + 1][col + 1] == "B") &&
                    damiera[row+2][col+2] == " "{
                
                damiera[row][col] = " "; // Cancello la posizione iniziale
                damiera[row+1][col+1] = " "; // Cancello la pedina avversaria mangiata
                damiera[row+2][col+2] = "N"; // Setto la nuova posizione della pedina

                mossa_scelta = logic::stampa_mossa(row, col).await + " " + &logic::stampa_mossa(row+2, col+2).await;

                cattura = true;
                break; // Ho mangiato ed esco dal for
            }
            else if ((row as i32) + 2 <= 7 && (col as i32) - 2 >= 0) && 
                    (damiera[row + 1][col - 1] == "b" || damiera[row + 1][col - 1] == "B") &&
                    damiera[row+2][col-2] == " "{
                
                damiera[row][col] = " "; // Cancello la posizione iniziale
                damiera[row+1][col-1] = " "; // Cancello la pedina avversaria mangiata
                damiera[row+2][col-2] = "N"; // Setto la nuova posizione della pedina

                mossa_scelta = logic::stampa_mossa(row, col).await + " " + &logic::stampa_mossa(row+2, col-2).await;

                cattura = true;
                break; // Ho mangiato ed esco dal for
            }
        }
    }

    // Mangio con la prima pedina disponibile
    if pedine.len() != 0 && cattura == false{

        for n in 0..pedine.len() {
            
            row = pedine[n][0];
            col = pedine[n][1];

            if ((row as i32) + 2 <= 7 && (col as i32) + 2 <= 7) && 
               damiera[row + 1][col + 1] == "b" &&
               damiera[row+2][col+2] == " "{
                
                damiera[row][col] = " "; // Cancello la posizione iniziale
                damiera[row+1][col+1] = " "; // Cancello la pedina avversaria mangiata
                damiera[row+2][col+2] = logic::dama("n", row+2).await; // Setto la nuova posizione della pedina e controllo se ho fatto dama

                mossa_scelta = logic::stampa_mossa(row, col).await + " " + &logic::stampa_mossa(row+2, col+2).await;

                cattura = true;
                break; // Ho mangiato ed esco dal for
            }
            else if ((row as i32) + 2 <= 7 && (col as i32) - 2 >= 0) && 
                    damiera[row + 1][col - 1] == "b" &&
                    damiera[row+2][col-2] == " "{
                
                damiera[row][col] = " "; // Cancello la posizione iniziale
                damiera[row+1][col-1] = " "; // Cancello la pedina avversaria mangiata
                damiera[row+2][col-2] = logic::dama("n", row+2).await; // Setto la nuova posizione della pedina e controllo se ho fatto dama

                mossa_scelta = logic::stampa_mossa(row, col).await + " " + &logic::stampa_mossa(row+2, col-2).await;

                cattura = true;
                break; // Ho mangiato ed esco dal for
            }
        }
    }

    // Se non ho mangiato faccio una mossa in maniera casuale
    if cattura == false{
        let n_pedine = pedine.len() + dame.len();
        let mut rng = SmallRng::from_entropy();
        let mut scelta = rng.gen_range(0..n_pedine);
        let mut mossa = false;
        let mut continua = true;

        if scelta < dame.len(){

            while continua == true || scelta != dame.len() {
                
                row = dame[scelta][0];
                col = dame[scelta][1];

                if ((row as i32) - 1 >= 0 && (col as i32) - 1 >= 0) &&
                damiera[row-1][col-1] == " " {
                    
                    damiera[row][col] = " "; // Cancello la posizione iniziale
                    damiera[row-1][col-1] = "N"; // Setto la nuova posizione della pedina

                    mossa_scelta = logic::stampa_mossa(row, col).await + " " + &logic::stampa_mossa(row-1, col-1).await;

                    mossa = true;
                    break; // Ho fatto una mossa ed esco dal for
                }
                else if ((row as i32) - 1 >= 0 && (col as i32) + 1 <= 7) &&
                        damiera[row-1][col+1] == " "{
                    
                    damiera[row][col] = " "; // Cancello la posizione iniziale
                    damiera[row-1][col+1] = "N"; // Setto la nuova posizione della pedina

                    mossa_scelta = logic::stampa_mossa(row, col).await + " " + &logic::stampa_mossa(row-1, col+1).await;

                    mossa = true;
                    break; // Ho fatto una mossa ed esco dal for
                }
                else if ((row as i32) + 1 <= 7 && (col as i32) + 1 <= 7) &&
                        damiera[row+1][col+1] == " "{
                    
                    damiera[row][col] = " "; // Cancello la posizione iniziale
                    damiera[row+1][col+1] = "N"; // Setto la nuova posizione della pedina

                    mossa_scelta = logic::stampa_mossa(row, col).await + " " + &logic::stampa_mossa(row+1, col+1).await;

                    mossa = true;
                    break; // Ho fatto una mossa ed esco dal for
                }
                else if ((row as i32) + 1 <= 7 && (col as i32) - 1 >= 0) &&
                        damiera[row+1][col-1] == " "{
                    
                    damiera[row][col] = " "; // Cancello la posizione iniziale
                    damiera[row+1][col-1] = "N"; // Setto la nuova posizione della pedina

                    mossa_scelta = logic::stampa_mossa(row, col).await + " " + &logic::stampa_mossa(row+1, col-1).await;

                    mossa = true;
                    break; // Ho fatto una mossa ed esco dal for
                }
                else {

                    if scelta == dame.len() - 1 {
                        continua = false;
                        scelta = 0;
                    }
                    else {
                        scelta += 1;
                    }
                }
            }
        }

        if mossa == false {
            // Reimposto la mossa su false
            continua = true;
            // Resetto la scelta considerando solo le pedine
            scelta = rng.gen_range(0..pedine.len());

            while continua == true || scelta != pedine.len() {
                
                row = pedine[scelta][0];
                col = pedine[scelta][1];

                if ((row as i32) + 1 <= 7 && (col as i32) + 1 <= 7) &&
                damiera[row+1][col+1] == " "{

                    damiera[row][col] = " "; // Cancello la posizione iniziale
                    damiera[row+1][col+1] = logic::dama("n", row+1).await; // Setto la nuova posizione della pedina e controllo se ho fatto dama

                    mossa_scelta = logic::stampa_mossa(row, col).await + " " + &logic::stampa_mossa(row+1, col+1).await;
                    
                    break; // Ho fatto una mossa ed esco dal for
                }
                else if ((row as i32) + 1 <= 7 && (col as i32) - 1 >= 0) &&
                        damiera[row+1][col-1] == " "{

                    damiera[row][col] = " "; // Cancello la posizione iniziale
                    damiera[row+1][col-1] = logic::dama("n", row+1).await; // Setto la nuova posizione della pedina e controllo se ho fatto dama

                    mossa_scelta = logic::stampa_mossa(row, col).await + " " + &logic::stampa_mossa(row+1, col-1).await;

                    break; // Ho fatto una mossa ed esco dal for
                }
                else {

                    if scelta == pedine.len() - 1 {
                        continua = false;
                        scelta = 0;
                    }else {
                        scelta += 1;    
                    }
                }
            }
        }
    }

    // Ritorno la damiera sia che ho eseguito una mossa che non
    (mossa_scelta, damiera)
}
