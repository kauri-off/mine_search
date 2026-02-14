import { useState, ChangeEvent, KeyboardEvent } from "react";
import { useNavigate } from "react-router-dom";

function ServerSearchBar() {
  const [serverIp, setServerIp] = useState("");
  const navigate = useNavigate();

  const handleInputChange = (event: ChangeEvent<HTMLInputElement>) => {
    setServerIp(event.target.value);
  };

  const handleKeyPress = (event: KeyboardEvent<HTMLInputElement>) => {
    if (event.key === "Enter") {
      handleSearch();
    }
  };

  const handleButtonClick = () => {
    handleSearch();
  };

  const handleSearch = () => {
    if (serverIp) {
      navigate("/server/" + serverIp);
    }
  };

  return (
    <div className="input-group mb-3">
      <input
        type="text"
        className="form-control"
        placeholder="Server ip"
        value={serverIp}
        onChange={handleInputChange}
        onKeyDown={handleKeyPress}
      />
      <button
        className="btn btn-outline-primary"
        type="button"
        id="button-addon2"
        onClick={handleButtonClick}
      >
        Open page
      </button>
    </div>
  );
}

export default ServerSearchBar;
