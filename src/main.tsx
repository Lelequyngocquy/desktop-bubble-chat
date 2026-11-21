import React, { Component, ReactNode, ErrorInfo } from "react";
import ReactDOM from "react-dom/client";
import "./i18n";
import App from "./App";
import { feLog } from "./services/loggerService";
import "./index.css";

interface ErrorBoundaryProps {
  children: ReactNode;
}

interface ErrorBoundaryState {
  hasError: boolean;
  error: Error | null;
  componentStack: string | null;
}

class RootErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = { hasError: false, error: null, componentStack: null };
  }
  static getDerivedStateFromError(error: Error): Partial<ErrorBoundaryState> {
    return { hasError: true, error };
  }
  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error("[RootErrorBoundary] Unhandled UI error:", error, errorInfo);
    this.setState({ componentStack: errorInfo.componentStack || null });
    try {
      feLog.error(
        `[RootErrorBoundary] ${error.message}\nStack: ${error.stack}\nComponents: ${errorInfo.componentStack}`,
      );
    } catch (logErr) {
      console.warn("[RootErrorBoundary] Failed to forward error to backend logger:", logErr);
    }
  }
  handleReload = () => {
    window.location.reload();
  };
  render() {
    if (this.state.hasError) {
      return (
        <div className="flex flex-col items-center justify-center h-screen w-screen bg-background-app text-content-primary p-8 select-none font-sans overflow-auto">
          <div className="max-w-xl w-full bg-background-surface border border-border rounded-xl p-6 shadow-md flex flex-col gap-4 text-center">
            <div className="w-12 h-12 rounded-full bg-status-errorBg text-status-error flex items-center justify-center mx-auto text-xl font-bold border border-status-errorBorder">
              !
            </div>
            <h2 className="text-lg font-semibold text-content-primary">UI Error Occurred</h2>
            <p className="text-sm text-status-error font-mono break-words text-left bg-status-errorBg p-3 rounded border border-status-errorBorder">
              {this.state.error?.message || "Unknown error occurred while loading UI."}
            </p>
            {this.state.componentStack && (
              <pre className="text-xs text-content-secondary font-mono text-left bg-background-subtle p-3 rounded border border-border max-h-48 overflow-auto select-text">
                {this.state.componentStack}
              </pre>
            )}
            <div className="flex gap-3 justify-center mt-2">
              <button
                type="button"
                onClick={this.handleReload}
                className="px-4 py-2 bg-primary hover:bg-primary-foreground text-content-inverse rounded-lg text-sm font-medium transition-colors cursor-pointer"
              >
                Reload Application
              </button>
            </div>
          </div>
        </div>
      );
    }
    return this.props.children;
  }
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <RootErrorBoundary>
      <App />
    </RootErrorBoundary>
  </React.StrictMode>,
);
