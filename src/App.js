import React, { useState, useEffect } from "react";
import { invoke } from '@tauri-apps/api/tauri';

function App() {
  const [employeeName, setEmployeeName] = useState("");
  const [employeeCode, setEmployeeCode] = useState("");
  const [loggedIn, setLoggedIn] = useState(false);
  const [timeElapsed, setTimeElapsed] = useState(0);  // Time in seconds
  const [activeApps, setActiveApps] = useState([]);

  // Function to login and start time tracking
  const login = () => {
    invoke("start_time_tracking").then(() => {
      setLoggedIn(true);
      setTimeElapsed(0);  // Reset timer on login
    });
  };

  // Function to logout and stop time tracking
  const logout = () => {
    invoke("stop_time_tracking").then(() => {
      invoke("save_csv_and_pack_to_tar", {
        employeeName: employeeName,
        employeeCode: employeeCode,
        timeElapsed: timeElapsed,
        apps: activeApps
      }).then(() => {
        invoke("send_tar_to_aws");
        setLoggedIn(false);
      });
    });
  };

  // Fetch active applications every second
  const fetchActiveApps = () => {
    invoke("get_active_apps").then(apps => {
      setActiveApps(apps);
    });
  };

  // Timer to track time and fetch active apps periodically
  useEffect(() => {
    let interval;
    if (loggedIn) {
      interval = setInterval(() => {
        setTimeElapsed((prev) => prev + 1);
        fetchActiveApps();
      }, 1000);
    }
    return () => clearInterval(interval);
  }, [loggedIn]);

  return (
    <div className="App">
      <header>
        <h1>Employee Monitoring App</h1>

        {/* Input fields for employee details */}
        <div>
          <label>
            Employee Name:
            <input type="text" value={employeeName} onChange={(e) => setEmployeeName(e.target.value)} disabled={loggedIn} />
          </label>
        </div>
        <div>
          <label>
            Employee Code:
            <input type="text" value={employeeCode} onChange={(e) => setEmployeeCode(e.target.value)} disabled={loggedIn} />
          </label>
        </div>

        {/* Login/Logout buttons */}
        <div>
          <button onClick={login} disabled={loggedIn || !employeeName || !employeeCode}>Login</button>
          <button onClick={logout} disabled={!loggedIn}>Logout</button>
        </div>

        {/* Display real-time time elapsed */}
        <h2>Time Elapsed: {timeElapsed} seconds</h2>

        {/* Display active applications in real-time */}
        <h3>Active Applications</h3>
        <ul>
          {activeApps.map((app, index) => <li key={index}>{app}</li>)}
        </ul>
      </header>
    </div>
  );
}

export default App;
