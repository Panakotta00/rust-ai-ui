use rust_bert::gpt2::GPT2Generator;
use rust_bert::pipelines::generation_utils::{GenerateOptions, LanguageGenerator};

fn main() {
	let model = GPT2Generator::new(Default::default()).unwrap();

	let input_context_1 = "The dog is";
	let input_context_2 = "The cat was";

	let generate_options = GenerateOptions {
		max_length: Some(600),
		..Default::default()
	};

	println!("Ready!");
	let output = model.generate(Some(&[input_context_1, input_context_2]), Some(generate_options));
	println!("Output: {output:?}");
}
