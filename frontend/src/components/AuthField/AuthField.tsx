import { useState, ChangeEvent } from "react";
import { AuthFieldProps } from "./AuthField.types";

const AuthField = ({ callback }: AuthFieldProps) => {
  const [password, setPassword] = useState<string>("");

  const handleSubmit = (e: ChangeEvent) => {
    e.preventDefault();
    callback(password);
  };

  const handlePasswordChange = (e: ChangeEvent<HTMLInputElement>) => {
    setPassword(e.target.value);
  };

  return (
    <div className="container p-4">
      <div className="row justify-content-center">
        <div className="col-md-6">
          <form onSubmit={handleSubmit}>
            <div className="mb-3">
              <label htmlFor="passwordInput" className="form-label">
                Enter Password:
              </label>
              <input
                type="password"
                id="passwordInput"
                className="form-control"
                value={password}
                onChange={handlePasswordChange}
                placeholder="Enter your password"
                required
              />
            </div>
            <button type="submit" className="btn btn-primary w-100">
              Submit
            </button>
          </form>
        </div>
      </div>
    </div>
  );
};

export default AuthField;
