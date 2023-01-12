import "./App.css";
import { invoke } from "@tauri-apps/api";

function App() {
	let greeting = function () {
		invoke("greet", { name: "Tauri" }).then((res) => {
			console.log(res);
		});
	};

	return (
		<div className="App">
			<h1>Trader bot simulation</h1>
			<div className="card">
				<button onClick={greeting}>Start!</button>
			</div>
		</div>
	);
}

export default App;
