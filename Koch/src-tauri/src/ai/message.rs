use chrono::Utc;
use rig::{
    client::{CompletionClient, ProviderClient},
    completion::{Chat, Prompt},
    providers::openai::{self, completion::Message, responses_api::Role},
    tool::Tool,
};
use std::{fs, sync::Mutex};
use tauri::ipc::RuntimeCapability;

use crate::{
    analyzer::analyzer::{AiChatMessage, AiChatMessageRole, LocalMessage, LocalMessageRole},
    engine::fen::translate_fen_for_model,
    server::server::ServerState,
};

#[tauri::command]
pub async fn send_llm_request(
    state: tauri::State<'_, Mutex<ServerState<'_>>>,
    msg: String,
) -> Result<(String, i32), String> {
    // capture user-sent time at function start
    let user_sent_at = Utc::now().to_rfc3339();

    // 1. Load System Prompt (No lock needed)
    let system_prompt = fs::read_to_string("src/ai/systemprompt.txt")
        .map_err(|e| format!("Failed to read system prompt: {}", e))?;

    let openai_client = openai::Client::from_env();
    let koch_ai = openai_client
        .agent("gpt-4o")
        .preamble(&system_prompt)
        .build();
    let history = {
        let state = state.lock().unwrap();
        state.analyzer_controller.chat_history.chat_messages.clone()
    };
    let (current_ply, board_context) = {
        let state = state.lock().unwrap();
        let pv = state.analyzer_controller.last_pv.clone();
        let (pv_data, pv_best_move) = {
            match pv {
                Some(pv) => (
                    format!("{}", &pv),
                    format!("{}", &pv.best_first_move().unwrap_or("No best move".into())),
                ),
                None => (
                    "No engine line data ".into(),
                    "No best move provided by engine".into(),
                ),
            }
        };
        let board_context = format!(
            "###Board Context###\nPlayer color: White\nBoard in FEN: {}\nBoard in viusal format\n {} \n###Engine evals##\n {} \nBest Move: {}\n Main Threat: {}\n ###User Prompt###\n {}",
            state.analyzer_controller.get_fen(),
            translate_fen_for_model(&state.analyzer_controller.get_fen()),
            pv_data,
            pv_best_move,
            if state.analyzer_controller.last_threat.is_some() {state.analyzer_controller.last_threat.clone().unwrap() } else {"No Threat".into()},
            msg

        );

        (state.analyzer_controller.current_ply, board_context)
    };
    println!("{}", &board_context);
    let response = koch_ai
        .chat(
            board_context,
            history
                .iter()
                .map(|lm| rig::completion::Message::from(lm.clone()))
                .collect::<Vec<rig::completion::Message>>(),
        )
        .await
        .expect("could not send message");
    let koch_sent_at = Utc::now().to_rfc3339();
    let _update_chat = {
        let mut state = state.lock().unwrap();
        let current_chat = &mut state.analyzer_controller.chat_history;
        current_chat.chat_messages.extend(vec![
            LocalMessage {
                role: LocalMessageRole::User,
                content: msg.clone(),
                move_index: current_ply as isize,
                sent_at: user_sent_at,
            },
            LocalMessage {
                role: LocalMessageRole::Assistent,
                content: response.clone(),
                move_index: current_ply as isize,
                sent_at: koch_sent_at,
            },
        ]);
        match current_chat.save() {
            Ok(_) => println!("chat saved"),
            Err(_) => println!("Failed to save chat"),
        };
    };
    // Only lock the state to get current_ply, then drop the guard before await
    // Add user message to chat history

    // Get response from LLM

    // Add assistant response to chat history

    Ok((response, current_ply))
}
