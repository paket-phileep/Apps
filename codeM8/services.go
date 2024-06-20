package main

import (
	"fmt"
	"io/ioutil"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"strconv"
	"strings"
	"sync"
	"syscall"

	"gopkg.in/yaml.v2"
)

// Configuration struct for services
type Config struct {
	Services map[string]Service `yaml:"services"`
}

// Service struct represents a service configuration
type Service struct {
	Command   string `yaml:"command"`
	Directory string `yaml:"directory"`
}

// Global variables
var (
	servicePIDs = make(map[string]int)
	mutex       sync.Mutex
)

// StartOrStopServices starts or stops services based on the configuration
func StartOrStopServices(config Config) {
	for serviceName, service := range config.Services {
		fmt.Printf("Checking status of service %s...\n", serviceName)

		pidFile := filepath.Join(service.Directory, "service.pid")

		if pidBytes, err := ioutil.ReadFile(pidFile); err == nil {
			pidStr := strings.TrimSpace(string(pidBytes))
			if pid, err := strconv.Atoi(pidStr); err == nil {
				if processExists(pid) {
					fmt.Printf("%s is already running with PID %d. Stopping...\n", serviceName, pid)
					terminateProcess(pid)
					os.Remove(pidFile)
				} else {
					fmt.Printf("Stale PID file found for %s. Removing %s.\n", serviceName, pidFile)
					os.Remove(pidFile)
				}
			}
		}

		fmt.Printf("Starting service %s...\n", serviceName)
		fmt.Printf("Command: %s\n", service.Command)
		fmt.Printf("Directory: %s\n", service.Directory)

		cmd := exec.Command("sh", "-c", service.Command)
		cmd.Dir = service.Directory

		if err := cmd.Start(); err != nil {
			fmt.Printf("Failed to start %s: %v\n", serviceName, err)
		} else {
			mutex.Lock()
			servicePIDs[serviceName] = cmd.Process.Pid
			mutex.Unlock()

			pidStr := strconv.Itoa(cmd.Process.Pid)
			if err := ioutil.WriteFile(pidFile, []byte(pidStr), 0644); err != nil {
				fmt.Printf("Failed to write PID file %s: %v\n", pidFile, err)
			} else {
				fmt.Printf("%s service started with PID %d.\n", serviceName, cmd.Process.Pid)
			}
		}
	}
}

// HTTP handler to handle /kill requests
func handleKillRequest(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "Invalid request method", http.StatusBadRequest)
		return
	}

	if err := r.ParseForm(); err != nil {
		http.Error(w, "Failed to parse form data", http.StatusBadRequest)
		return
	}

	pidStr := r.Form.Get("pid")
	portStr := r.Form.Get("port")
	serviceName := r.Form.Get("service")

	if pidStr != "" {
		if pid, err := strconv.Atoi(pidStr); err == nil {
			if processExists(pid) {
				if err := terminateProcess(pid); err == nil {
					fmt.Fprintf(w, "Killed service with PID %d\n", pid)
				} else {
					http.Error(w, fmt.Sprintf("Failed to kill service with PID %d", pid), http.StatusInternalServerError)
				}
				return
			}
		}
		http.Error(w, fmt.Sprintf("Invalid PID: %s", pidStr), http.StatusBadRequest)
		return
	}

	if portStr != "" {
		http.Error(w, "Port-based service killing is not implemented in this example", http.StatusNotImplemented)
		return
	}

	if serviceName != "" {
		mutex.Lock()
		defer mutex.Unlock()

		if pid, ok := servicePIDs[serviceName]; ok {
			if processExists(pid) {
				if err := terminateProcess(pid); err == nil {
					fmt.Fprintf(w, "Killed service %s with PID %d\n", serviceName, pid)
				} else {
					http.Error(w, fmt.Sprintf("Failed to kill service %s with PID %d", serviceName, pid), http.StatusInternalServerError)
				}
				return
			} else {
				delete(servicePIDs, serviceName)
				http.Error(w, fmt.Sprintf("Service %s was not running", serviceName), http.StatusNotFound)
				return
			}
		}

		http.Error(w, fmt.Sprintf("Service %s not found", serviceName), http.StatusNotFound)
		return
	}

	http.Error(w, "No valid parameters found in request", http.StatusBadRequest)
}

// Check if a process with given PID exists
func processExists(pid int) bool {
	process, err := os.FindProcess(pid)
	if err != nil {
		return false
	}
	err = process.Signal(os.Signal(syscall.Signal(0)))
	return err == nil
}

// Terminate a process with given PID
func terminateProcess(pid int) error {
	process, err := os.FindProcess(pid)
	if err != nil {
		return err
	}
	return process.Kill()
}

func main() {
	// Read configuration from workspaces.yaml
	config, err := readConfig("workspaces.yaml")
	if err != nil {
		fmt.Printf("Failed to read config: %v\n", err)
		return
	}

	StartOrStopServices(config)

	http.HandleFunc("/kill", handleKillRequest)
	fmt.Println("Starting HTTP server on port 8080...")
	if err := http.ListenAndServe(":8080", nil); err != nil {
		fmt.Printf("Failed to start HTTP server: %v\n", err)
	}
}

// Function to read YAML configuration
func readConfig(filename string) (Config, error) {
	var config Config
	yamlFile, err := ioutil.ReadFile(filename)
	if err != nil {
		return config, err
	}

	err = yaml.Unmarshal(yamlFile, &config)
	if err != nil {
		return config, err
	}

	return config, nil
}
