import "./App.css";
import { useState, useRef } from "react";
import { app, invoke } from "@tauri-apps/api";
import { listen } from "@tauri-apps/api/event";
import React from "react";
function App() {
	const inputRef = useRef("");
	// create a ref to a mutable object
	const traderRef = useRef({
		name: "",
		budget: 0,
		goods: [],
		earnings: 0,
	});
	const [reRender, setReRender] = useState(false);
	const [outputs, setOutputs] = useState({ value: [] });
	const [output, setOutput] = useState({ value: "" });

	let sendOutput = function () {
		invoke("js2rs", { window: window.__TAURI__.invoke });
	};

	// add listener on mount
	React.useEffect(() => {
		const unlisten = listen("rs2js", (payload) => {
			console.log("received", payload);
			inputRef.current = payload.payload;

			// traderRef.current = {
			// 	name: payload.payload.name,
			// 	budget: payload.payload.budget,
			// 	earnings: payload.payload.earnings,
			// };
			// // transform the payload goods object in array
			// let goods = [];
			// for (let key in payload.payload.goods) {
			// 	goods.push(payload.payload.goods[key]);
			// }
			// traderRef.current.goods = goods;

			// console.log("traderRef", traderRef.current);
			setReRender(!reRender);
		});

		return () => {
			unlisten.then((f) => f());
		};
	}, [reRender]);
	let closeProcess = function () {
		invoke("close_process");
	};

	let clearAll = function () {
		console.log("clearing");
		inputRef.current = "";
		setOutputs({ value: [] });
	};

	return (
		<div style={{ width: "100% ", display: "inline" }}>
			<div>
				<button onClick={sendOutput}>Send</button>
				<button onClick={clearAll}>Reset</button>
				<button onClick={closeProcess}>close</button>
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
				<div style={{ display: "inline-block" }}>
					<h2>Inputs: {inputRef.current}</h2>
					{/* {inputs.map((input) => (
						<li key={input.timestamp}>
							{input.timestamp} {input.message}
						</li>
					))} */}
				</div>
			</div>
		</div>
	);
}

export default App;
