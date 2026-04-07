use crate::models::*;
use std::collections::HashMap;

pub fn calculate_stats(projects: &[Project]) -> GlobalStats {
    let mut gs = GlobalStats {
        projects: HashMap::new(),
        overall: ProjectStats::default(),
    };
    gs.overall.name = "Overall".to_string();

    for proj in projects {
        let mut ps = ProjectStats::default();
        ps.name = proj.name.clone();

        for sess in &proj.sessions {
            for msg in &sess.messages {
                let model = msg.model.as_deref().unwrap_or("gemini-1.5-pro");
                let pricing = get_pricing(model);
                
                if let Some(tokens) = &msg.tokens {
                    let cost = (tokens.input as f64 / 1_000_000.0 * pricing.0) +
                               (tokens.output as f64 / 1_000_000.0 * pricing.1);
                    
                    ps.cost += cost;
                    ps.total_tokens += tokens.total;
                    ps.input += tokens.input;
                    ps.output += tokens.output;
                    *ps.models.entry(model.to_string()).or_insert(0) += 1;
                    ps.token_history.push((msg.timestamp, tokens.total));

                    gs.overall.cost += cost;
                    gs.overall.total_tokens += tokens.total;
                    gs.overall.input += tokens.input;
                    gs.overall.output += tokens.output;
                    *gs.overall.models.entry(model.to_string()).or_insert(0) += 1;
                    gs.overall.token_history.push((msg.timestamp, tokens.total));
                }
            }
        }
        ps.token_history.sort_by_key(|h| h.0);
        gs.projects.insert(proj.name.clone(), ps);
    }
    gs.overall.token_history.sort_by_key(|h| h.0);
    gs
}

pub fn get_pricing(model: &str) -> (f64, f64) {
    if model.contains("flash") { (0.075, 0.30) }
    else { (3.50, 10.50) }
}
