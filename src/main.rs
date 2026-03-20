// Copyright (c) 2025 Cloudflight. All rights reserved.

slint::include_modules!();
use std::fs::read_to_string;
use std::error::Error;
use std::rc::Rc;
use std::sync::mpsc::{Sender, RecvTimeoutError};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use slint::{Model, SharedString, Timer, TimerMode};
use std::thread;
use std::vec::Vec;
use std::{ops::Deref, sync::mpsc};
use websocket::{ClientBuilder, Message};


#[derive(Debug, Deserialize, Clone)]
struct AppConfig {
    pub websocket_address: String
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
#[serde(rename_all = "camelCase")]
enum CoreServiceState {
    Idle,
    Detecting,
    Detected,
    Producing,
    Done,
}

struct SystemState {
    core_service_state: CoreServiceState,
    current_error: String,
    abort_requested: bool
}

enum UiMessage {
    AbortProduction,
    StartProduction(i32),
    GetProducts,
    ReconnectWebsocket
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct SerializableProduct {
    icon_id: i32,
    name: String,
    displayed_name: String,
    product_id: i32,
    fun_fact: String,
    ingredients: Vec<SerializableIngredient>,
    feasible: bool,
    default: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct SerializableIngredient {
    name: String,
    percentage: f32,
    color: String,
}

#[derive(Debug, Clone)]
struct SendableProduct {
    icon_id: i32,
    name: SharedString,
    fun_fact: SharedString,
    product_id: i32,
    ingredients: Vec<Ingredient>,
    can_manually_select: bool
}

impl From<Product> for SendableProduct {
    fn from(p: Product) -> Self {
        SendableProduct {
            icon_id: p.iconId,
            name: p.name,
            fun_fact: p.funFact,
            product_id: p.productId,
            ingredients: p.ingredients.iter().collect(),
            can_manually_select: p.canManuallySelect
        }
    }
}

impl Into<Product> for SendableProduct {
    fn into(self) -> Product {
        Product {
            iconId: self.icon_id,
            name: SharedString::from(self.name.as_str()),
            funFact: SharedString::from(self.fun_fact.as_str()),
            productId: self.product_id,
            ingredients: slint::ModelRc::new(slint::VecModel::from(self.ingredients)),
            canManuallySelect: self.can_manually_select
        }
    }
}

fn read_config_file() -> Result<String, Box<dyn Error>> {
    let mut config_path = std::env::current_exe()?;
    config_path.set_file_name("/usr/share/coffee-app/config.yaml");
    Ok(read_to_string(config_path)?)
}

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;

    let config: AppConfig = if let Ok(config_string) = read_config_file() {
        serde_yaml::from_str(&config_string)?
    } else {
        println!("No config file found, loading defaults.");
        AppConfig {
            websocket_address: String::from("ws://localhost:3000/frontend")
        }
    };
    println!("Expecting websocket server at: {}", config.websocket_address);

    let state_advance_timer = Rc::new(Timer::default());
    let system_state = Arc::new(RwLock::new(SystemState {
        core_service_state: CoreServiceState::Idle,
        current_error: String::from(""),
        abort_requested: false
    }));

    let sender_tx = setup_callbacks(ui.as_weak(), state_advance_timer.clone(), system_state.clone(), config.clone());

    // Run the client in not blocking thread.
    thread::spawn({
        let ui_weak_handler = ui.as_weak();
        let system_state = system_state.clone();
        let sender_tx = sender_tx.clone();
        move || {
            loop {
                let client_ws_events = ClientBuilder::new(&config.websocket_address)
                    .unwrap()
                    .connect_insecure();

                if let Ok(mut client) = client_ws_events {
                    println!("Connected response receiver");
                    client.stream_ref().set_read_timeout(Some(Duration::from_secs(10)))
                        .expect("could not set read timeout on socket");
                    let mut products: Vec<Product> = Vec::new();
                    sender_tx.send(UiMessage::GetProducts).unwrap();

                    for message in client.incoming_messages() {
                        match message {
                            Ok(result) => {
                                let result: Message = result.into();

                                // Convert message payload to string
                                let mut message_string: String = "".to_string();
                                let payload = result.payload;
                                message_string.push_str(std::str::from_utf8(payload.deref()).unwrap());
                                parse_websocket_message(&message_string, ui_weak_handler.clone(), &mut products, &system_state, sender_tx.clone());
                                if products.is_empty() {
                                    sender_tx.send(UiMessage::GetProducts).unwrap();
                                }
                            }
                            Err(error) => {
                                println!("error on receive: {}", error);
                                println!("resetting connections...");
                                sender_tx.send(UiMessage::ReconnectWebsocket).unwrap();
                                break;
                            }
                        }
                    }
                } else {
                    println!("No connection to core service, expected at: {}", config.websocket_address);
                    sender_tx.send(UiMessage::ReconnectWebsocket).unwrap();
                    thread::sleep(Duration::from_secs(1));
                }
            }
        }
    });

    let elapsed_clock = Timer::default();
    elapsed_clock.start(TimerMode::Repeated, Duration::from_millis(50), {
        let ui = ui.as_weak();
        move || {
            let ui = ui.upgrade().unwrap();
            let old_elapsed_ms = ui.get_elapsed_since_state_change();
            if old_elapsed_ms < 60 * 1000 {
                ui.set_elapsed_since_state_change(old_elapsed_ms + 50);
            }
        }
    });

    // Start the ui, this will clock the main thread, part behind this call will not be executed
    ui.run()?;
    // ensure the timer does not get moved out of the main thread
    drop(state_advance_timer);
    Ok(())
}

// Subscribe to all callbacks of the application
fn setup_callbacks(ui_weak: slint::Weak<AppWindow>, state_advance_timer: Rc<Timer>, system_state: Arc<RwLock<SystemState>>, config: AppConfig) -> Sender<UiMessage> {
    let ui = ui_weak.unwrap();
    ui.on_change_state({
        let system_state = system_state.clone();
        let state_advance_timer = state_advance_timer.clone();
        let ui_handle = ui.as_weak();
        move |state| {
            if state == UiState::ManualSelection {
                update_ui_state(state, ui_handle.clone(), system_state.clone(), state_advance_timer.clone());
            } else {
                // FIXME nicer transitions from and to ManualSelection
                // (hint: manual selection should be orthogonal to UiState)
                let new_ui_state = get_ui_state_for_system_state(&system_state.read().unwrap());
                update_ui_state(new_ui_state, ui_handle.clone(), system_state.clone(), state_advance_timer.clone());
            }
        }
    });

    let (tx, rx) = mpsc::channel();

    ui.on_start_production({
        let tx = tx.clone();
        move |id: i32| { tx.send(UiMessage::StartProduction(id)).unwrap(); }
    });

    ui.on_abort_production({
        let tx = tx.clone();
        move || { tx.send(UiMessage::AbortProduction).unwrap(); }
    });

    ui.on_toggled_error_overlay({
        let state_advance_timer = state_advance_timer.clone();
        let ui = ui.as_weak();
        move |overlay_is_shown: bool| {
            let ui = ui.upgrade().unwrap();
            if ui.get_state() == UiState::DetectedCup {
                // Stop / Restart auto production timer when the overlay is shown
                if overlay_is_shown {
                    state_advance_timer.stop();
                } else {
                    start_auto_production_timer(state_advance_timer.clone(), ui.as_weak());
                }
            }
        }
    });

    // Thread to send messages. Used to restart and wait for a websocket connection after it get lost
    thread::spawn(move || {
        // Loop to restart a websocket
        loop {
            let client_ui_events = ClientBuilder::new(&config.websocket_address)
                .unwrap()
                .connect_insecure();

            if let Ok(mut client) = client_ui_events {
                println!("Connected request sender");
                // Wait for production/abort calls from the UI until a message send get us an error.
                loop {
                    match rx.recv_timeout(Duration::from_secs(2)) {
                        Ok(UiMessage::StartProduction(id)) => {
                            let message = Message::text(get_production_message(id));
                            if !client.send_message(&message).is_ok() {
                                println!("Error at send production");
                                break;
                            }
                        }
                        Ok(UiMessage::AbortProduction) => {
                            let message_part_one =
                                r#"{"jsonrpc":"2.0","method": "abortProduction" ,"params": {}}"#;
                            let message = Message::text(message_part_one);
                            if !client.send_message(&message).is_ok() {
                                println!("Error at send abort");
                                break;
                            }
                            let mut system_state = system_state.write().unwrap();
                            system_state.abort_requested = true;
                        }
                        Ok(UiMessage::GetProducts) => {
                            let message = r#"{"jsonrpc":"2.0","method": "getProducts" ,"params": {}, "id": "getProducts"}"#;
                            if !client.send_message(&Message::text(message)).is_ok() {
                                println!("Error requesting products");
                                break;
                            }
                            println!("Sent get products request: {}", message);
                        }
                        Ok(UiMessage::ReconnectWebsocket) => {
                            println!("resetting sender connection...");
                            break;
                        }
                        Err(RecvTimeoutError::Timeout) => {
                            let message = r#"{"jsonrpc":"2.0","method": "ping" ,"params": {}, "id": "ping"}"#;
                            if !client.send_message(&Message::text(message)).is_ok() {
                                println!("error sending ping");
                                break;
                            }
                        }
                        Err(RecvTimeoutError::Disconnected) => {
                            panic!("Channel to sender disconnected!");
                        }
                    }
                }
            } else {
                thread::sleep(Duration::from_millis(500));
            }
        }
    });

    tx
}

// Parse the response from the product REST request to a List of Products
fn parse_product_list(data: &Vec<Value>) -> Vec<Product> {
    let mut ps: Vec<Product> = Vec::new();
    for v in data.iter() {
        let p: SerializableProduct = match serde_json::from_value(v.clone()) {
            Ok(product) => product,
            Err(error) => {
                println!("Could not parse product: {}", error);
                return Vec::default();
            }
        };
        let mut ig: Vec<Ingredient> = Vec::new();

        for i in p.ingredients.iter() {
            let color_string = i.color.as_str();
            let ingredient = Ingredient {
                name: SharedString::from(i.name.as_str()),
                percentage: i.percentage,
                // Parse hex color string to color
                color: slint::Color::from_rgb_u8(
                    u8::from_str_radix(&color_string[1..3], 16).unwrap(),
                    u8::from_str_radix(&color_string[3..5], 16).unwrap(),
                    u8::from_str_radix(&color_string[5..7], 16).unwrap(),
                ),
            };
            ig.push(ingredient);
        }

        let product = Product {
            iconId: p.icon_id,
            name: SharedString::from(p.displayed_name.as_str()),
            funFact: SharedString::from(p.fun_fact.as_str()),
            productId: p.product_id,
            ingredients: slint::ModelRc::new(slint::VecModel::from(ig.clone())),
            canManuallySelect: p.feasible && p.default
        };
        ps.push(product);
    }
    return ps;
}

// must be called on ui thread (main thread)
fn update_ui_state(new_state: UiState, ui_weak: slint::Weak<AppWindow>, system_state: Arc<RwLock<SystemState>>, state_advance_timer: Rc<Timer>) {
    let ui = ui_weak.upgrade().unwrap();
    let old_state = ui.get_state();
    if old_state == new_state {
        return;
    }

    // Transition from detection to idle indicates detection failure
    let new_state = if new_state == UiState::Idle &&
        (old_state == UiState::DetectionStarted || old_state == UiState::DetectedCup) {
        UiState::DetectionFailed
    } else {
        new_state
    };

    // Defer advancing state for screens that have a minimum show time...
    if old_state != UiState::DetectedCup && state_advance_timer.running()
     // ... but always change immediately to these states:
     && (new_state != UiState::InProduction && new_state != UiState::ManualSelection &&
         new_state != UiState::DetectionFailed && new_state != UiState::DetectionStarted) {
        return;
    }

    ui.set_state(new_state);
    ui.set_elapsed_since_state_change(0);

    if new_state != UiState::InProduction {
        // reset abort state
        let mut system_state = system_state.write().unwrap();
        system_state.abort_requested = false;
    }

    // Start state advance timer according to state.
    let timer = state_advance_timer.clone();
    match new_state {
        UiState::DetectionStarted => {
            // Show for at least 2s
            state_advance_timer.start(
                TimerMode::SingleShot,
                Duration::from_secs(2),
                move || {
                    let new_ui_state = get_ui_state_for_system_state(&system_state.read().unwrap());
                    update_ui_state(new_ui_state, ui_weak.clone(), system_state.clone(), timer.clone());
            });
        }

        UiState::DetectionFailed => {
            // Show for 10s
            state_advance_timer.start(
                TimerMode::SingleShot,
                Duration::from_secs(10),
                move || {
                    let new_ui_state = get_ui_state_for_system_state(&system_state.read().unwrap());
                    update_ui_state(new_ui_state, ui_weak.clone(), system_state.clone(), timer.clone());
            });
        }

        UiState::DetectedCup => {
            if !ui.get_show_error_overlay() {
                start_auto_production_timer(state_advance_timer, ui.as_weak());
            } else {
                state_advance_timer.stop();
            }
        }

        UiState::FinishedAnimation => {
            // show animation for 5s before showing fun fact screen
            state_advance_timer.start(
                TimerMode::SingleShot,
                Duration::from_secs(5),
                move || update_ui_state(UiState::FinishedFunfact, ui_weak.clone(), system_state.clone(), timer.clone())
            );
        }

        UiState::FinishedFunfact => {
            // show funfact for at least 5s
            state_advance_timer.start(
                TimerMode::SingleShot,
                Duration::from_secs(5),
                move || {
                    let is_not_same_state = system_state.read().unwrap().core_service_state != CoreServiceState::Done;
                    if is_not_same_state {
                        let new_ui_state = get_ui_state_for_system_state(&system_state.read().unwrap());
                        update_ui_state(new_ui_state, ui_weak.clone(), system_state.clone(), timer.clone());
                    }
            });
        }

        _ => { state_advance_timer.stop(); }
    }
}

// must be called on ui thread (main thread)
fn update_coreservice_state(new_state: CoreServiceState, ui_weak: &slint::Weak<AppWindow>, system_state: Arc<RwLock<SystemState>>) {
    {
        let mut system_state = system_state.write().unwrap();
        system_state.core_service_state = new_state;
    }

    // update ui state if manual selection is not currently shown
    let ui = ui_weak.upgrade().unwrap();
    if ui.get_state() != UiState::ManualSelection || new_state == CoreServiceState::Producing {
        let new_ui_state = get_ui_state_for_system_state(&system_state.read().unwrap());
        ui.invoke_change_state(new_ui_state);
    }
}

// must be called on ui thread (main thread)
fn start_auto_production_timer(state_advance_timer: Rc<Timer>, ui_weak: slint::Weak<AppWindow>) {
    let ui = ui_weak.upgrade().unwrap();
    state_advance_timer.start(TimerMode::SingleShot, Duration::from_secs(5), move || {
        ui.invoke_start_production(ui.get_current_production().productId);
    });
}

fn parse_websocket_message(received: &String, ui_weak: slint::Weak<AppWindow>, products: &mut Vec<Product>, system_state: &Arc<RwLock<SystemState>>, sender_tx: Sender<UiMessage>) {

    let message: Value = match serde_json::from_str(received.as_str()) {
        Ok(message) => message,
        Err(error) => {
            println!("Failed to parse as JSON: {:?}", error);
            return;
        }
    };
    if message["id"] != "ping" {
        println!("Got JSON from websocket: {}", received);
    }

    // Handle message which updates the state
    if message["method"] == "updateState" {
        let state = if let Ok(state) = CoreServiceState::deserialize(&message["params"]["state"]) {
            state
        } else {
            println!("Failed to parse state in {}", received);
            return;
        };

        println!("New system state: {:?}", state);
        ui_weak.upgrade_in_event_loop({
            let system_state = system_state.clone();
            move |ui| { update_coreservice_state(state, &ui.as_weak(), system_state) }
        }).unwrap();

        if state == CoreServiceState::Idle {
            // Make sure product list is up-to-date
            sender_tx.send(UiMessage::GetProducts).unwrap();
        }

        let production_id = message["params"]["productId"].as_i64();
        if production_id.is_none() {
            return;
        }
        let product_id = production_id.unwrap() as i32;
        let found_product = products
            .iter()
            .find(|&p| p.productId == product_id);
        if let Some(product) = found_product {
            let sendable_product: SendableProduct = product.clone().into();
            ui_weak.upgrade_in_event_loop(move |ui| {
                ui.set_current_production(sendable_product.into());

                // Update the ingredients by the now producing product.
                let ingredients = ui.get_ingredients();
                let product_ingredients = ui
                    .get_products()
                    .iter()
                    .find(|p| p.productId == ui.get_current_production().productId)
                    .unwrap()
                    .ingredients;
                for i in 0..ingredients.row_count() {
                    let product_ingredient = product_ingredients
                        .iter()
                        .find(|j| j.name == ingredients.row_data(i).unwrap().name);

                    // Update ingredient, prevent to set totally new ingredient object, otherwise animation will not work
                    let mut original_ingredient = ingredients.row_data(i).unwrap();
                    if product_ingredient.is_some() {
                        original_ingredient.percentage =
                            product_ingredient.unwrap().percentage;
                        ingredients.set_row_data(i, original_ingredient);
                    } else {
                        original_ingredient.percentage = 0.0;
                        ingredients.set_row_data(i, original_ingredient);
                    }
                }

                println!("Set this ingredients: ");
                for i in ui.get_ingredients().clone().iter() {
                    println!("{}, {}", i.name, i.percentage)
                }
            }).unwrap();
        }
    } else if message["method"] == "updateErrorDialog" {
        let new_error = String::from(
            if let Some(text) = message["params"]["text"].as_str() {
                text
            } else {
                ""
            });
        {
            let mut system_state = system_state.write().unwrap();
            system_state.current_error = new_error;
        }
        let error_type = if system_state.read().unwrap().current_error.is_empty() {
            ErrorType::None
        } else {
            ErrorType::Generic
        };
        ui_weak.upgrade_in_event_loop({
            move |ui| {
                ui.set_current_error(error_type);
                if error_type != ErrorType::None {
                    ui.set_show_error_overlay(true);
                    ui.invoke_toggled_error_overlay(true);
                }
            }
        }).unwrap();

        // Handles response message of get products
    } else if message["id"] == "getProducts" {
        // Set all products
        let product_string = message["result"].as_array();
        if product_string.is_none() {
            println!("Failed to parse {:?}", received);
            return;
        }
        *products = parse_product_list(product_string.unwrap());

        // Search in products for all ingredients and set them
        let mut ingredients: Vec<Ingredient> = Vec::new();
        for p in products.iter() {
            for i in 0..p.ingredients.row_count() {
                let ingredient = p.ingredients.row_data_tracked(i).unwrap();

                if ingredients
                    .iter()
                        .find(|&j| j.name.contains(ingredient.name.as_str()))
                        .is_none()
                {
                    ingredients.push(ingredient);
                }
            }
        }

        let sendable_products: Vec<SendableProduct> = products.iter().map(|p| p.clone().into()).collect();
        ui_weak.upgrade_in_event_loop(move |ui| {
            let products: Vec<Product> = sendable_products.iter().map(|sp| Into::<Product>::into(sp.clone())).collect();
            let products = slint::ModelRc::new(slint::VecModel::from(products));
            let selectable_products = slint::FilterModel::new(products.clone(), |p| p.canManuallySelect);
            ui.set_products(products);
            ui.set_selectable_products(slint::ModelRc::new(selectable_products));
            ui.set_ingredients(slint::ModelRc::new(slint::VecModel::from(ingredients)));
        }).unwrap();
    } else if message["id"] != "ping" {
        println!("Got unexpected message {:?}", received);
    }
}

// Convert the states of the states JSON to state of the UI
fn get_ui_state_for_system_state(state: &SystemState) -> UiState {
    return match state.core_service_state {
        CoreServiceState::Idle => UiState::Idle,
        CoreServiceState::Detecting => UiState::DetectionStarted,
        CoreServiceState::Detected => UiState::DetectedCup,
        CoreServiceState::Producing => UiState::InProduction,
        CoreServiceState::Done => {
            if state.abort_requested {
                // Do not celebrate on abort
                UiState::FinishedFunfact
            } else {
                UiState::FinishedAnimation
            }
        }
    }
}

// Creates the JSON message to set the product to produce
fn get_production_message(id: i32) -> String {
    let message_part_one =
        r#"{"jsonrpc":"2.0","method": "selectProduct" ,"params": {"productId": "#;
    let message_part_two = r#"}}"#;
    let mut message: String = "".to_owned();
    message.push_str(message_part_one);
    message.push_str(id.to_string().as_str());
    message.push_str(message_part_two);
    return message;
}
