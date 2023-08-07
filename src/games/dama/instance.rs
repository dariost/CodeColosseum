use super::super::util::Player;
use super::bot;
use super::logic;
use crate::game;
use async_trait::async_trait;
use tracing::warn;
use rand::rngs::StdRng;
use std::collections::HashMap;
use tokio::io::{AsyncWriteExt, DuplexStream, WriteHalf};
use tokio::time::Duration;

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

        lnout2!(giocatore[0].output, "Avvio la partita di dama...");
        lnout2!(giocatore[1].output, "Avvio la partita di dama...");
        lnout2!(spectators, "Avvio la partita di dama...");

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

        // Inizzia sempre la partita il secondo giocatore che si conette
        _ = giocatore[0].output.write(("Sei i Bianchi\n").as_bytes()).await;
        _ = giocatore[1].output.write(("Sei i Neri\n").as_bytes()).await;
        _ = spectators.write(("Il giocatore ".to_owned() + &giocatore[0].name + " è i bianchi e il giocatore " + &giocatore[1].name + " è i neri.\n").as_bytes()).await;

        // Avvio il gioco
        while !logic::partita_in_corso(damiera.clone(), &mut giocatore, &mut spectators).await {
            
            if turno_binaco { // Bianchi
                _ = giocatore[0].output.write(("Turno bianco!\nE' il tuo turno.\n\n").as_bytes()).await;
                _ = giocatore[1].output.write(("Turno bianco!\n").as_bytes()).await;
                _ = spectators.write(("Turno bianco!\n").as_bytes()).await;

                // Controllo chi deve muovere le pedine bianche 
                if &giocatore[0].name == "ServerBot$0"
                {
                    // Faccio muovere le pedine binche al bot
                    damiera = bot::bot_bianco(damiera.clone(), &mut giocatore[1].output).await;

                }
                else {
                    // Verifico la validità del percorso dato dall'utente
                    percorso_valido = logic::verifica_percorso_bianco(damiera.clone(), &mut giocatore[0]).await;

                    // Aggiorno la damiera
                    damiera = logic::aggionra_damiera(percorso_valido.clone(), damiera.clone(), &mut giocatore, &mut spectators).await;
                }

                // Cambio il turno di gioco
                turno_binaco = false;
            }
            else { // Neri
                _ = giocatore[0].output.write(("Turno nero!\n").as_bytes()).await;
                _ = giocatore[1].output.write(("Turno nero!\nE' il tuo turno.\n\n").as_bytes()).await;
                _ = spectators.write(("Turno nero!\n").as_bytes()).await;

                // Controllo chi deve muovere le pedine nere 
                if &giocatore[1].name == "ServerBot$0"
                {
                    // Faccio muovere le pedine nere al bot
                    damiera = bot::bot_nero(damiera.clone(), &mut giocatore[0].output).await;

                }
                else {
                    // Verifico la validità del percorso dato dall'utente
                    percorso_valido = logic::verifica_percorso_nero(damiera.clone(), &mut giocatore[1]).await;

                    // Aggiorno la damiera
                    damiera = logic::aggionra_damiera(percorso_valido.clone(), damiera.clone(), &mut giocatore, &mut spectators).await;
                }
        
                // Cambio il turno di gioco
                turno_binaco = true;
            }
            
            // Stampo la damiera aggiornata
            logic::stampa_damiera(damiera.clone(), &mut giocatore, &mut spectators).await;
        }

        // Fine del gioco
         _ = giocatore[0].output.write(("Per iniziare una nuova partira riavvia il gioco ;)\n\n").as_bytes()).await;
         _ = giocatore[1].output.write(("Per iniziare una nuova partira riavvia il gioco ;)\n\n").as_bytes()).await;
         _ = spectators.write(("Per iniziare una nuova partira riavvia il gioco ;)\n\n").as_bytes()).await;
    }
}
