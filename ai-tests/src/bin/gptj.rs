use std::path::PathBuf;
use rust_bert::gpt_j::{GptJConfigResources, GptJMergesResources, GptJVocabResources};
use rust_bert::pipelines::common::ModelType;
use rust_bert::pipelines::conversation::{ConversationConfig, ConversationManager, ConversationModel};
use rust_bert::pipelines::text_generation::{TextGenerationConfig, TextGenerationModel};
use rust_bert::pipelines::token_classification::LabelAggregationOption::Mode;
use rust_bert::resources::{LocalResource, RemoteResource};
use tch::Device;

fn main() {
	let config_resource = Box::new(LocalResource::from(PathBuf::from(
		"../pygmalion-6b/config.json"
	)));

	let vocab_resource = Box::new(LocalResource::from(PathBuf::from(
		"../pygmalion-6b/vocab.json"
	)));

	let merges_resource = Box::new(LocalResource::from(PathBuf::from(
		"../pygmalion-6b/merges.txt"
	)));

	let model_resource = Box::new(LocalResource::from(PathBuf::from(
		"../pygmalion-6b/rust_model.ot",
	)));

	let generation_config = TextGenerationConfig {
		model_type: ModelType::GPTJ,
		model_resource,
		config_resource,
		vocab_resource,
		merges_resource: Some(merges_resource),
		min_length: 10,
		max_length: Some(20),
		do_sample: false,
		early_stopping: true,
		num_beams: 1,
		num_return_sequences: 1,
		device: Device::cuda_if_available(),
		..Default::default()
	};

	let mut model = TextGenerationModel::new(generation_config).unwrap();
	model.half();

	println!("Ready!");

	let prompts = ["Hello."];
	let output = model.generate(&prompts, None);

	println!("{output:?}");
}
