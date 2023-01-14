import "./App.css";
import { useState, useRef } from "react";
import { invoke } from "@tauri-apps/api";
import { listen } from "@tauri-apps/api/event";
import React from "react";
function App() {
	const inputRef = useRef(new Array());
	const [reRender, setReRender] = useState(false);
	const [outputs, setOutputs] = useState({ value: [] });
	const [output, setOutput] = useState({ value: "" });

	let sendOutput = function () {
		setOutputs({
			value: [
				...outputs.value,
				{ timestamp: new Date(Date.now()), message: output.value },
			],
		});
		invoke("js2rs", { message: output.value });
	};

	// add listener on mount
	React.useEffect(() => {
		const unlisten = listen("rs2js", (payload) => {
			inputRef.current.push({
				timestamp: new Date(Date.now()),
				message: payload.payload,
			});
			setReRender(!reRender);
		});

		return () => {
			unlisten.then((f) => f());
		};
	}, [reRender]);

	let clearAll = function () {
		console.log("clearing");
		inputRef.current = [];
		setOutputs({ value: [] });
	};

	return (
		<div style={{ width: "100% ", display: "inline" }}>
			<div>
				<button onClick={sendOutput}>Send</button>
				<button onClick={clearAll}>Reset</button>
			</div>
			<div>
				<input
					type="text"
					value={output.value}
					onChange={(e) => setOutput({ value: e.target.value })}
				/>
			</div>
			{/* display divs next to each other */}
			<div style={{ display: "inline", width: "100%" }}>
				<div style={{ display: "inline-block", margin: "20px" }}>
					<h2>Outputs</h2>
					{outputs.value.map((output) => (
						<li key={output.timestamp.getTime()}>
							{output.timestamp.toISOString()} {output.message}
						</li>
					))}
				</div>
				<div style={{ display: "inline-block" }}>
					<h2>Inputs: {inputRef.length}</h2>
					{/* {inputs.map((input) => (
						<li key={input.timestamp}>
							{input.timestamp} {input.message}
						</li>
					))} */}
					{inputRef.current.map((input) => (
						<li key={input.timestamp.getTime()}>
							{input.timestamp.toISOString()} {input.message}
						</li>
					))}
				</div>
			</div>
		</div>
	);
}

export default App;
