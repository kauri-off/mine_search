import { useState } from "react";
import { AuthFieldProps } from "./AuthField.types";

function AuthField({ callback }: AuthFieldProps) {
  const [pin, setPin] = useState("");

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    callback(pin); // Передаём PIN через callback
  };

  return (
    <form onSubmit={handleSubmit} className="p-3">
      <div className="mb-3">
        <label htmlFor="pinInput" className="form-label">
          Введите PIN:
        </label>
        <input
          type="password"
          className="form-control"
          id="pinInput"
          value={pin}
          onChange={(e) => setPin(e.target.value)}
          placeholder="PIN-код"
        />
      </div>
      <button type="submit" className="btn btn-primary">
        Отправить
      </button>
    </form>
  );
}

export default AuthField;
