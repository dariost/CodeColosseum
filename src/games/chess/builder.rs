// Import delle dipendenze e dei moduli necessari per il gioco
use super::bot::Bot;
use super::instance::Instance;
use crate::game;
use crate::games;
use async_trait::async_trait;
use games::util::arg;
use rand::rngs::{OsRng, StdRng};
use rand::SeedableRng;
use std::collections::HashMap;
use tokio::time::Duration;

// Costanti di default per il timeout e il ritmo del gioco
const DEFAULT_TIMEOUT: f64 = 90.0;
const DEFAULT_PACE: f64 = 1.5;

// Definizione della struttura Builder che sarà responsabile per la costruzione delle istanze del gioco
#[derive(Debug)]
pub(crate) struct Builder {}

impl Builder {
    // Metodo che restituisce una nuova istanza di Builder
    pub(crate) fn new() -> Box<dyn game::Builder> {
        Box::new(Builder {})
    }
}

// Implementazione del trait game::Builder per il Builder
#[async_trait]
impl game::Builder for Builder {
    // Metodo che restituisce il nome del gioco "royalur"
    fn name(&self) -> &str {
        "chess"
    }

    // Metodo asincrono che legge il contenuto del file "description.md" e lo restituisce come una stringa
    async fn description(&self) -> String {
        String::from(include_str!("description.md"))
    }

    // Metodo asincrono che genera un'istanza del gioco
    async fn gen_instance(
        &self,
        param: &mut game::Params,
        args: HashMap<String, String>,
    ) -> Result<Box<dyn game::Instance>, String> {
        // Controllo e validazione del numero di giocatori
        param.players = match param.players {
            Some(2) => Some(2),
            Some(x) => return Err(format!("Cannot create game with {} players", x)),
            None => Some(2),
        };

        // Controllo e impostazione del timeout del gioco
        param.timeout = param.timeout.or(Some(DEFAULT_TIMEOUT));

        // Calcolo del ritmo del gioco leggendo l'argomento "pace" da args
        let pace = match arg(&args, "pace", DEFAULT_PACE) {
            Ok(x) if x < 0.0 || x > 30.0 => return Err(format!("Invalid pace")),
            Ok(x) => x,
            Err(x) => return Err(format!("Invaid pace: {}", x)),
        };

        // Inizializzazione del generatore di numeri casuali (PRNG)
        let rng = match StdRng::from_rng(OsRng) {
            Ok(x) => x,
            Err(x) => return Err(format!("Cannot initialize PRNG: {}", x)),
        };

        // Restituzione dell'istanza del gioco incapsulata in un Box
        Ok(Box::new(Instance {
            timeout: Duration::from_secs_f64(param.timeout.expect("Cannot fail")),
            pace: Duration::from_secs_f64(pace),
            rng: rng,
        }))
    }

    // Metodo asincrono che genera un bot per il gioco
    async fn gen_bot(&self) -> Box<dyn game::Bot> {
        Box::new(Bot {})
    }
}
