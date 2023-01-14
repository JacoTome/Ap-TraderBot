import "./App.css";
import { useState } from "react";
import { invoke } from "@tauri-apps/api";
import { listen } from "@tauri-apps/api/event";
function App() {
	const [inputs, setInputs] = useState({ value: [] });
	const [outputs, setOutputs] = useState({ value: [] });
	const [output, setOutput] = useState({ value: "" });

	let sendOutput = function () {
		console.log("js: js2rs: " + output.value);
		setOutputs({
			value: [
				...outputs.value,
				{ timestamp: Date.now(), message: output.value },
			],
		});
		invoke("js2rs", { message: output.value });
	};

	const listen = async () => {
		await listen("rs2js", (payload) => {
			console.log("js: rs2js: " + payload.message);
			setInputs({
				value: [
					...inputs.value,
					{ timestamp: Date.now(), message: payload.message },
				],
			});
		});
	};

	return (
		<>
			<div>
				HELLO
				<button onClick={sendOutput}>Send</button>
			</div>
			<div>
				<input
					type="text"
					value={output.value}
					onChange={(e) => setOutput({ value: e.target.value })}
				/>
			</div>
			<div>
				{outputs.value.map((output) => (
					<div key={output.timestamp}>
						{output.timestamp} {output.message}
					</div>
				))}
			</div>
		</>
	);
}

export default App;
