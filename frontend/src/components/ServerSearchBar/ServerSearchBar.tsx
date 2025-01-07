import { useState, ChangeEvent, KeyboardEvent } from "react";
import { useNavigate } from "react-router-dom";

function ServerSearchBar() {
  const [serverIp, setServerIp] = useState("");
  const navigate = useNavigate();

  // Обработчик для изменения значения в поле
  const handleInputChange = (event: ChangeEvent<HTMLInputElement>) => {
    setServerIp(event.target.value);
  };

  // Обработчик для нажатия Enter
  const handleKeyPress = (event: KeyboardEvent<HTMLInputElement>) => {
    if (event.key === "Enter") {
      handleSearch();
    }
  };

  // Обработчик для нажатия на кнопку
  const handleButtonClick = () => {
    handleSearch();
  };

  // Функция для выполнения поиска
  const handleSearch = () => {
    if (serverIp) {
      // Перенаправление на страницу с IP-адресом
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
        onKeyDown={handleKeyPress} // Добавляем обработчик нажатия клавиш
      />
      <button
        className="btn btn-outline-primary"
        type="button"
        id="button-addon2"
        onClick={handleButtonClick} // Обработчик клика на кнопку
      >
        Open page
      </button>
    </div>
  );
}

export default ServerSearchBar;
