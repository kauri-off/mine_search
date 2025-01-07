import { BrowserRouter, Route, Routes } from "react-router-dom";
import Home from "./routes/Home";
import Server from "./routes/Server";
import Players from "./routes/Players";

function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<Home />}></Route>
        <Route path="/players" element={<Players />}></Route>
        <Route path="/server/:ip" element={<Server />}></Route>
      </Routes>
    </BrowserRouter>
  );
}

export default App;
