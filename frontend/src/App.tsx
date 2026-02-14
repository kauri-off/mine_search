import { BrowserRouter, Route, Routes } from "react-router-dom";
import Home from "./routes/Home";
import Server from "./routes/Server";
import Stats from "./routes/Stats";
import Auth from "./routes/Auth";

function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<Home />}></Route>
        <Route path="/server/:ip" element={<Server />}></Route>
        <Route path="/stats" element={<Stats />}></Route>
        <Route path="/auth" element={<Auth />}></Route>
      </Routes>
    </BrowserRouter>
  );
}

export default App;
