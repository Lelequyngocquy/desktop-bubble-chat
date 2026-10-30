import React, { Component, ReactNode, ErrorInfo } from "react";
import ReactDOM from "react-dom/client";
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
        `[RootErrorBoundary] ${error.message}\nStack: ${error.stack}\nComponents: ${errorInfo.componentStack}`
      );
    } catch {}
  }

  handleReload = () => {
    window.location.reload();
  };

  render() {
    if (this.state.hasError) {
      return (
        <div className="flex flex-col items-center justify-center h-screen w-screen bg-[#EEF2F9] text-[#202733] p-8 select-none font-sans overflow-auto">
          <div className="max-w-xl w-full bg-white border border-[#E2E7EF] rounded-xl p-6 shadow-md flex flex-col gap-4 text-center">
            <div className="w-12 h-12 rounded-full bg-red-100 text-red-600 flex items-center justify-center mx-auto text-xl font-bold">
              !
            </div>
            <h2 className="text-lg font-semibold text-[#202733]">Đã xảy ra sự cố giao diện</h2>
            <p className="text-sm text-red-600 font-mono break-words text-left bg-red-50 p-3 rounded border border-red-200">
              {this.state.error?.message || "Lỗi không xác định khi tải giao diện."}
            </p>
            {this.state.componentStack && (
              <pre className="text-xs text-[#6B7483] font-mono text-left bg-gray-50 p-3 rounded border border-gray-200 max-h-48 overflow-auto select-text">
                {this.state.componentStack}
              </pre>
            )}
            <div className="flex gap-3 justify-center mt-2">
              <button
                onClick={this.handleReload}
                className="px-4 py-2 bg-[#0866FF] hover:bg-[#0759DB] text-white rounded-lg text-sm font-medium transition-colors"
              >
                Tải lại ứng dụng
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
