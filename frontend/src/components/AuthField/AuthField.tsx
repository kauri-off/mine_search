import { useState } from "react";
import { AuthFieldProps } from "./AuthField.types";

function AuthField({ callback }: AuthFieldProps) {
  const [password, setPassword] = useState("");

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    callback(password); // Pass the password via callback
  };

  return (
    <form onSubmit={handleSubmit} className="p-3">
      <div className="mb-3">
        <label htmlFor="passwordInput" className="form-label">
          Enter Password:
        </label>
        <input
          type="password"
          className="form-control"
          id="passwordInput"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          placeholder="Password"
        />
      </div>
      <button type="submit" className="btn btn-primary">
        Submit
      </button>
    </form>
  );
}

export default AuthField;
