import React from "react";
import {
  ArrowUpCircle,
  CheckCircle2,
  Download,
  ExternalLink,
  Info,
  Pause,
  RefreshCw,
  RotateCcw,
  ShieldCheck,
  Sparkles,
} from "lucide-react";
import { TabHeader, SwitchButton } from "../common";
import { formatBytes, useAutoUpdater } from "../../hooks/useAutoUpdater";
import { systemService } from "../../services/systemService";

export const UpdatesWorkspace: React.FC = () => {
  const {
    status,
    currentVersion,
    availableVersion,
    releaseDate,
    releaseNotes,
    downloadedBytes,
    totalBytes,
    progressPercent,
    downloadSpeed,
    errorMessage,
    lastCheckedTime,
    autoCheckOnStartup,
    checkForUpdates,
    startDownload,
    cancelDownload,
    installAndRelaunch,
    setAutoCheckOnStartup,
  } = useAutoUpdater();

  const isChecking = status === "checking";
  const isDownloading = status === "downloading";
  const isDownloaded = status === "downloaded";
  const isInstalling = status === "installing";
  const isAvailable = status === "available";
  const isUpToDate = status === "up-to-date";

  const handleOpenGitHubReleases = () => {
    systemService.openExternalUrl("https://github.com/Lelequyngocquy/desktop-bubble-chat/releases");
  };

  return (
    <div className="flex-1 min-w-0 min-h-0 h-full flex flex-col bg-background-workspace overflow-hidden select-none">
      {/* 1. Header chung */}
      <TabHeader
        eyebrow="HỆ THỐNG & PHIÊN BẢN"
        title="Cập nhật ứng dụng"
        description="Kiểm tra phiên bản mới nhất, quản lý cài đặt và nâng cấp tự động với độ tin cậy tuyệt đối."
        rightSlot={
          <div className="flex items-center gap-2 px-3 py-1.5 bg-background-surface border border-border rounded-pill text-small shadow-xs">
            <span
              className={`w-2.5 h-2.5 rounded-full ${
                isAvailable || isDownloaded
                  ? "bg-primary animate-pulse"
                  : isChecking
                  ? "bg-amber-500 animate-spin"
                  : "bg-status-success"
              }`}
            />
            <span className="text-content-primary font-semibold">
              v{currentVersion}
            </span>
          </div>
        }
      />

      {/* 2. Nội dung cuộn chính */}
      <main className="flex-1 min-h-0 overflow-y-auto px-8 py-6 flex flex-col gap-6 custom-scrollbar">
        {/* Card 1: Trạng thái tổng quan & Nút hành động chính */}
        <section className="bg-background-surface border border-border rounded-panel p-6 shadow-xs flex flex-col gap-6">
          <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
            <div className="flex items-start gap-4">
              <div
                className={`w-14 h-14 rounded-2xl flex items-center justify-center shrink-0 shadow-sm ${
                  isAvailable || isDownloaded
                    ? "bg-primary text-white shadow-primary/20"
                    : isUpToDate
                    ? "bg-emerald-50 text-emerald-600 border border-emerald-200"
                    : isChecking
                    ? "bg-blue-50 text-primary border border-blue-200"
                    : "bg-slate-100 text-content-secondary border border-border"
                }`}
              >
                {isChecking ? (
                  <RefreshCw className="w-7 h-7 animate-spin" />
                ) : isAvailable ? (
                  <Sparkles className="w-7 h-7 animate-bounce" />
                ) : isDownloaded ? (
                  <Download className="w-7 h-7 text-white" />
                ) : isUpToDate ? (
                  <CheckCircle2 className="w-7 h-7" />
                ) : (
                  <ArrowUpCircle className="w-7 h-7" />
                )}
              </div>

              <div className="flex flex-col gap-1">
                <div className="flex items-center gap-2.5 flex-wrap">
                  <h3 className="text-lg font-bold text-content-primary m-0">
                    {isChecking
                      ? "Đang kiểm tra bản cập nhật..."
                      : isDownloading
                      ? `Đang tải bản cập nhật v${availableVersion}...`
                      : isDownloaded
                      ? `Đã tải xong bản cập nhật v${availableVersion}`
                      : isInstalling
                      ? "Đang khởi chạy bộ cài đặt..."
                      : isAvailable
                      ? `Đã có bản cập nhật mới: v${availableVersion}`
                      : isUpToDate
                      ? "Bạn đang dùng phiên bản mới nhất"
                      : "Sẵn sàng kiểm tra bản cập nhật"}
                  </h3>

                  {isAvailable && (
                    <span className="px-2 py-0.5 rounded-pill text-xs font-semibold bg-primary text-white shadow-xs">
                      Bản mới
                    </span>
                  )}
                  {isDownloaded && (
                    <span className="px-2 py-0.5 rounded-pill text-xs font-semibold bg-emerald-600 text-white shadow-xs">
                      Sẵn sàng kích hoạt
                    </span>
                  )}
                </div>

                <p className="text-small text-content-secondary m-0">
                  {isChecking
                    ? "Đang kết nối đến GitHub Releases CDN để quét dữ liệu..."
                    : isDownloading
                    ? `Tiến trình: ${progressPercent}% · ${formatBytes(downloadedBytes)} / ${formatBytes(totalBytes)} (${downloadSpeed})`
                    : isDownloaded
                    ? "Gói cài đặt đã được tải xuống và kiểm tra toàn vẹn. Nhấn 'Cài đặt & Khởi động lại' để hoàn tất."
                    : isAvailable
                    ? `Phiên bản v${availableVersion} đã sẵn sàng. Bạn có thể tải ngay bây giờ hoặc tiếp tục sử dụng.`
                    : isUpToDate
                    ? `DesktopBubbleChat v${currentVersion} là phiên bản ổn định nhất hiện hành.`
                    : "Nhấn nút bên dưới để kiểm tra các tính năng mới từ GitHub."}
                </p>

                {releaseDate && (
                  <span className="text-caption text-content-muted mt-0.5">
                    Ngày phát hành: {new Date(releaseDate).toLocaleDateString("vi-VN")}
                  </span>
                )}
              </div>
            </div>

            {/* Các nút hành động chính */}
            <div className="flex items-center gap-2.5 flex-wrap shrink-0">
              {/* Nút hành động tương ứng với trạng thái */}
              {isAvailable && (
                <button
                  onClick={startDownload}
                  className="flex items-center gap-2 px-4 py-2 bg-primary hover:bg-primary-hover text-white rounded-control text-small font-semibold shadow-xs transition-colors"
                >
                  <Download className="w-4 h-4" />
                  <span>Tải bản cập nhật</span>
                </button>
              )}

              {isDownloading && (
                <button
                  onClick={cancelDownload}
                  className="flex items-center gap-2 px-4 py-2 bg-rose-50 hover:bg-rose-100 text-rose-600 border border-rose-200 rounded-control text-small font-semibold transition-colors"
                >
                  <Pause className="w-4 h-4" />
                  <span>Dừng tải</span>
                </button>
              )}

              {/* Nút Khởi động lại: CHỈ hiển thị khi có bản mới được fetch và người dùng đã tải thành công */}
              {isDownloaded && (
                <button
                  onClick={installAndRelaunch}
                  disabled={isInstalling}
                  title="Khởi động lại ứng dụng để kích hoạt phiên bản mới vừa tải"
                  className="flex items-center gap-2 px-4 py-2 bg-emerald-600 hover:bg-emerald-700 text-white rounded-control text-small font-semibold shadow-xs transition-colors disabled:opacity-60 animate-pulse"
                >
                  <RotateCcw className={`w-4 h-4 ${isInstalling ? "animate-spin" : ""}`} />
                  <span>{isInstalling ? "Đang áp dụng..." : "Khởi động lại để cập nhật"}</span>
                </button>
              )}

              {/* Nút Kiểm tra bản mới */}
              {!isDownloading && !isInstalling && (
                <button
                  onClick={() => checkForUpdates(false)}
                  disabled={isChecking}
                  className={`flex items-center gap-2 px-3.5 py-2 border border-border hover:bg-background-app text-content-primary rounded-control text-small font-medium transition-colors ${
                    isChecking ? "opacity-60 cursor-not-allowed" : ""
                  }`}
                >
                  <RefreshCw
                    className={`w-4 h-4 text-content-secondary ${
                      isChecking ? "animate-spin text-primary" : ""
                    }`}
                  />
                  <span>{isChecking ? "Đang quét..." : "Kiểm tra bản mới"}</span>
                </button>
              )}
            </div>
          </div>

          {/* Thanh tiến trình tải mượt mà (chỉ hiển thị khi đang tải hoặc đã tải xong) */}
          {(isDownloading || isDownloaded) && (
            <div className="flex flex-col gap-2 pt-2 border-t border-border/60">
              <div className="flex items-center justify-between text-caption font-medium">
                <span className="text-content-secondary">
                  {isDownloaded
                    ? "Tải xuống hoàn tất 100%"
                    : `Đang tải: ${progressPercent}% · Tốc độ: ${downloadSpeed}`}
                </span>
                <span className="text-content-primary font-semibold">
                  {formatBytes(downloadedBytes)} / {formatBytes(totalBytes)}
                </span>
              </div>

              <div className="w-full h-3 bg-slate-100 rounded-pill overflow-hidden border border-border/40 p-0.5">
                <div
                  className={`h-full rounded-pill transition-all duration-300 ${
                    isDownloaded
                      ? "bg-emerald-500"
                      : "bg-gradient-to-r from-primary to-blue-400"
                  }`}
                  style={{ width: `${Math.max(4, progressPercent)}%` }}
                />
              </div>
            </div>
          )}

          {/* Thông báo lỗi nếu có */}
          {errorMessage && (
            <div className="p-3 bg-rose-50 border border-rose-200 rounded-control text-rose-700 text-small flex items-center justify-between gap-2">
              <div className="flex items-center gap-2">
                <Info className="w-4 h-4 shrink-0 text-rose-500" />
                <span>{errorMessage}</span>
              </div>
              <button
                onClick={() => checkForUpdates(false)}
                className="text-xs font-semibold text-rose-700 underline hover:no-underline shrink-0"
              >
                Thử lại
              </button>
            </div>
          )}

          {/* Hàng thông tin phụ: Lần kiểm tra cuối & Toggle Tự động */}
          <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 pt-3 border-t border-border/60 text-small">
            <div className="flex items-center gap-2 text-content-secondary">
              <span className="w-1.5 h-1.5 rounded-full bg-content-muted" />
              <span>
                Kiểm tra lần cuối:{" "}
                <strong className="text-content-primary font-medium">
                  {lastCheckedTime || "Chưa quét trong phiên này"}
                </strong>
              </span>
            </div>

            <div className="flex items-center gap-3">
              <span className="text-content-secondary">
                Tự động kiểm tra khi mở ứng dụng
              </span>
              <SwitchButton
                checked={autoCheckOnStartup}
                onChange={setAutoCheckOnStartup}
                ariaLabel="Tự động kiểm tra bản cập nhật khi mở app"
              />
            </div>
          </div>
        </section>

        {/* Card 2: Nhật ký phát hành (Changelog / Release Notes) */}
        <section className="bg-background-surface border border-border rounded-panel p-6 shadow-xs flex flex-col gap-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2.5">
              <div className="w-8 h-8 rounded-lg bg-blue-50 text-primary flex items-center justify-center">
                <Sparkles className="w-4 h-4" />
              </div>
              <h4 className="text-nav font-bold text-content-primary m-0">
                {isAvailable || isDownloaded
                  ? `Điểm mới trong bản phát hành v${availableVersion}`
                  : `Thông tin phiên bản hiện tại v${currentVersion}`}
              </h4>
            </div>

            <button
              onClick={handleOpenGitHubReleases}
              className="flex items-center gap-1.5 text-small text-primary hover:underline font-medium"
            >
              <span>Xem trên GitHub Releases</span>
              <ExternalLink className="w-3.5 h-3.5" />
            </button>
          </div>

          <div className="bg-background-app/70 border border-border/80 rounded-control p-4 text-small text-content-primary leading-relaxed whitespace-pre-wrap font-sans">
            {releaseNotes ? (
              releaseNotes
            ) : isAvailable ? (
              "Bản cập nhật bao gồm các cải tiến hiệu năng, tính ổn định và sửa lỗi tổng thể cho ứng dụng."
            ) : (
              `🎉 DesktopBubbleChat v${currentVersion} đã sẵn sàng phục vụ!\n\n` +
              "• Tích hợp cơ chế bong bóng nổi trên màn hình Windows độc lập và siêu nhẹ.\n" +
              "• Hỗ trợ đăng nhập và chuyển đổi linh hoạt nhiều tài khoản Messenger song song.\n" +
              "• Tự động ngủ đông giải phóng RAM (0 MB UI RAM) khi đóng về khay hệ thống.\n" +
              "• Cơ chế cập nhật tự động 1-click an toàn qua chữ ký mật mã Minisign."
            )}
          </div>
        </section>

        {/* Card 3: Thông số cài đặt & Môi trường hệ thống */}
        <section className="bg-background-surface border border-border rounded-panel p-6 shadow-xs flex flex-col gap-4">
          <div className="flex items-center gap-2.5">
            <div className="w-8 h-8 rounded-lg bg-slate-100 text-content-secondary flex items-center justify-center">
              <ShieldCheck className="w-4 h-4 text-primary" />
            </div>
            <div>
              <h4 className="text-nav font-bold text-content-primary m-0">
                Thông tin môi trường & Bảo mật cập nhật
              </h4>
              <p className="text-caption text-content-secondary m-0">
                Cơ chế cập nhật được bảo vệ bằng chữ ký mật mã chống can thiệp trái phép.
              </p>
            </div>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-3 pt-2">
            <div className="p-3 bg-background-app/50 border border-border/60 rounded-control flex flex-col gap-1">
              <span className="text-caption text-content-secondary">
                Định danh ứng dụng (App ID)
              </span>
              <span className="text-small font-semibold text-content-primary font-mono">
                com.desktopbubblechat.app
              </span>
            </div>

            <div className="p-3 bg-background-app/50 border border-border/60 rounded-control flex flex-col gap-1">
              <span className="text-caption text-content-secondary">
                Kênh phân phối (Distribution Channel)
              </span>
              <span className="text-small font-semibold text-content-primary">
                GitHub Releases (CDN Toàn cầu)
              </span>
            </div>

            <div className="p-3 bg-background-app/50 border border-border/60 rounded-control flex flex-col gap-1">
              <span className="text-caption text-content-secondary">
                Xác thực chữ ký (Signature Verification)
              </span>
              <span className="text-small font-semibold text-emerald-600 flex items-center gap-1.5">
                <CheckCircle2 className="w-3.5 h-3.5" />
                Minisign Cryptographic Ed25519 (.sig)
              </span>
            </div>

            <div className="p-3 bg-background-app/50 border border-border/60 rounded-control flex flex-col gap-1">
              <span className="text-caption text-content-secondary">
                Kiến trúc thực thi (Runtime Architecture)
              </span>
              <span className="text-small font-semibold text-content-primary">
                Tauri 2.0 · Rust Core · Windows x86_64
              </span>
            </div>
          </div>
        </section>
      </main>
    </div>
  );
};

export default UpdatesWorkspace;
