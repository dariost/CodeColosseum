use super::super::util::Player;
use super::logic;
use crate::game;
use async_trait::async_trait;
use tracing::warn;
use rand::rngs::StdRng;
use std::collections::HashMap;
use tokio::io::{AsyncWriteExt, DuplexStream, WriteHalf};
use tokio::time::{sleep_until, Duration, Instant};

#[derive(Debug)]
pub(crate) struct Instance {
    pub(crate) timeout: Duration,
    pub(crate) pace: Duration,
    pub(crate) rng: StdRng,
}

#[async_trait]
impl game::Instance for Instance {
    async fn start(&mut self, players: HashMap<String, DuplexStream>, mut spectators: WriteHalf<DuplexStream>,) {
        let mut giocatore = Player::from(players, &mut self.rng);
        assert_eq!(giocatore.len(), 2);

        // Invio i nomi dei giocatori
        for i in 0..2 {
            lnout2!(giocatore[0].output, &giocatore[i].name);
            lnout2!(giocatore[1].output, &giocatore[i].name);
            lnout2!(spectators, &giocatore[i].name);
        }

        _ = giocatore[0].output.write(("Avvio la partita di dama...\n").as_bytes()).await;
        _ = giocatore[1].output.write(("Avvio la partita di dama...\n").as_bytes()).await;
        _ = spectators.write(("Avvio la partita di dama...\n").as_bytes()).await;

        let mut damiera: Vec<Vec<&str>> = vec![vec![" ", "n", " ", "n", " ", "n", " ", "n"],
                                               vec!["n", " ", "n", " ", "n", " ", "n", " "],
                                               vec![" ", "n", " ", "n", " ", "n", " ", "n"],
                                               vec![" ", " ", " ", " ", " ", " ", " ", " "],
                                               vec![" ", " ", " ", " ", " ", " ", " ", " "],
                                               vec!["b", " ", "b", " ", "b", " ", "b", " "],
                                               vec![" ", "b", " ", "b", " ", "b", " ", "b"],
                                               vec!["b", " ", "b", " ", "b", " ", "b", " "]
                                              ];
        
        // Stampo la damiera
        logic::stampa_damiera(damiera.clone(), &mut giocatore, &mut spectators).await;

        // Setto chi inizzia a giocare
        let mut turno_binaco: bool = true;

        // Creo il vettore che conterrà il percorso valido
        let mut percorso_valido: Vec<String> = Vec::new();

        // Creao la stringa che conterrà la mossa da inviare all'avversario
        let mut mossa: String = String::new();

        // Inizia sempre la partita il secondo giocatore che si conette
        _ = giocatore[0].output.write(("Sei i Bianchi\n").as_bytes()).await;
        _ = giocatore[1].output.write(("Sei i Neri\n").as_bytes()).await;
        _ = spectators.write(("Il giocatore ".to_owned() + &giocatore[0].name + " è i bianchi e il giocatore " + &giocatore[1].name + " è i neri.\n").as_bytes()).await;

        // Avvio il gioco
        while !logic::partita_in_corso(damiera.clone(), &mut giocatore, &mut spectators, turno_binaco).await {
            
            let start = Instant::now();

            if turno_binaco { // Bianchi
                
                // Dico al giocatore cosa deve muovere
                lnout2!(giocatore[0].output, "Turno bianco!");
                _ = giocatore[1].output.write(("Attendi il tuo turno!\n").as_bytes()).await;
                _ = spectators.write(("Turno bianco!\n").as_bytes()).await;
                
                // Verifico la validità del percorso dato dall'utente
                percorso_valido = logic::verifica_percorso_bianco(damiera.clone(), &mut giocatore[0], self.timeout).await;
                
                // Controllo se c'è stato un abbandono della partita
                if percorso_valido[0] == "Err"{
                    _ = giocatore[1].output.write(("\nI binachi hanno abbandonato la partita.\nI neri vincono la partita!\n\n").as_bytes()).await;
                    break;
                }

                // Converto le mosse da numeriche a alfanumeriche
                for i in 0..percorso_valido.len() {

                    mossa += &(logic::conv_mossa_in_alfanum(&percorso_valido[i]).await + " ")
                }

                // Invio la mossa fatta all'avversario
                lnout2!(giocatore[1].output, mossa.clone());
                
                // Cambio il turno di gioco
                turno_binaco = false;
            }
            else { // Neri
                
                // Dico al giocatore cosa deve muovere
                lnout2!(giocatore[1].output, "Turno nero!");
                _ = giocatore[0].output.write(("Attendi il tuo turno!\n").as_bytes()).await;
                _ = spectators.write(("Turno nero!\n").as_bytes()).await;
                
                // Verifico la validità del percorso dato dall'utente
                percorso_valido = logic::verifica_percorso_nero(damiera.clone(), &mut giocatore[1], self.timeout).await;
                
                // Controllo se c'è stato un abbandono della partita
                if percorso_valido[0] == "Err"{
                    _ = giocatore[0].output.write(("\nI neri hanno abbandonato la partita.\nI bianchi vincono la partita!\n\n").as_bytes()).await;
                    break;
                }

                // Converto le mosse da numeriche a alfanumeriche
                for i in 0..percorso_valido.len() {

                    mossa += &(logic::conv_mossa_in_alfanum(&percorso_valido[i]).await + " ");
                }

                // Invio la mossa fatta all'avversario
                lnout2!(giocatore[0].output, mossa.clone());
                
                // Cambio il turno di gioco
                turno_binaco = true;
            }
            
            // Pulisco la variabile usata per passare la mossa all'avversario
            mossa.clear();

            // Aggiorno la damiera
            damiera = logic::aggionra_damiera(percorso_valido.clone(), damiera.clone()).await;

            // Stampo la damiera aggiornata
            logic::stampa_damiera(damiera.clone(), &mut giocatore, &mut spectators).await;

            // Faccio una pausa
            sleep_until(start + self.pace).await;
        }

        // Fine del gioco
         _ = giocatore[0].output.write(("Game Over ;)\n\n").as_bytes()).await;
         _ = giocatore[1].output.write(("Game Over ;)\n\n").as_bytes()).await;
         _ = spectators.write(("Game Over ;)\n\n").as_bytes()).await;
    }
}
