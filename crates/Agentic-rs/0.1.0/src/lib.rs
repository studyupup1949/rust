use ollama_rs::{
    generation::{completion::request::GenerationRequest, options::GenerationOptions},
    Ollama,
};

pub struct Cognitive {
    engine: Ollama,
    model: String,
}
pub struct Agent {
    name: String,
    persona: Message,
    cognitive_resource: Cognitive,
    memory: Vec<Message>,
    role: Role,
}

impl Agent {
    pub fn new(name: String, persona: Message, talent: Cognitive) -> Self {
        Self {
            name,
            persona,
            cognitive_resource: talent,
            memory: Vec::new(),
            role: Role::AGENT,
        }
    }
    pub async fn execute(&self, task: String) -> String {
        let text = format!("You are {:#?}.\nBackground of this task is {:#?} Please to following instruction: {:#?}\n ",self.persona,self.memory,task,);
        let prompt = Message::new(text.to_string(), Role::AGENT);
        self.cognitive_resource.reflect(prompt).await.to_string()
    }
}
#[derive(Debug)]
pub struct Message {
    content: String,
    role: Role,
}
impl Message {
    pub fn new(prompt: String, role: Role) -> Self {
        Self {
            content: prompt,
            role: role,
        }
    }
}
#[derive(Debug)]
pub enum Role {
    AGENT,
    USER,
    SYSTEM,
}
impl Cognitive {
    pub fn new(model: String) -> Self {
        Self {
            engine: Ollama::default(),
            model,
        }
    }

    pub async fn reflect(&self, prompt: Message) -> String {
        let options = GenerationOptions::default()
            .temperature(0.2)
            .repeat_penalty(1.5)
            .top_k(25)
            .top_p(0.25);

        let res = self
            .engine
            .generate(GenerationRequest::new(self.model.clone(), prompt.content).options(options))
            .await;

        match res {
            Ok(res) => res.response,
            Err(_) => "Unknown".to_string(),
        }
    }
}
