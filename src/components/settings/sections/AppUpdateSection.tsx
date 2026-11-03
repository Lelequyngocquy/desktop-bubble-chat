import React, { useEffect, useState, useCallback } from "react";
import {
  ArrowUpCircle,
  CheckCircle2,
  Download,
  ExternalLink,
  Info,
  Pause,
  RefreshCw,
  RotateCcw,
  Sparkles,
} from "lucide-react";
import { CommonLargeCard, SwitchButton } from "../../common";
import { formatBytes, useAutoUpdater } from "../../../hooks/useAutoUpdater";
import { systemService } from "../../../services/systemService";

interface GitHubReleaseData {
  tag: string;
  name: string;
  publishedAt: string;
  htmlUrl: string;
  body: string;
}

export const AppUpdateSection: React.FC = () => {
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
  const hasUpdate = isAvailable || isDownloading || isDownloaded;

  // Trạng thái quản lý Release Notes lấy trực tiếp từ GitHub (không hardcode)
  const [githubRelease, setGithubRelease] = useState<GitHubReleaseData | null>(
    null,
  );
  const [isLoadingNotes, setIsLoadingNotes] = useState(false);
  const [notesError, setNotesError] = useState<string | null>(null);

  const fetchGitHubReleaseNotes = useCallback(async () => {
    setIsLoadingNotes(true);
    setNotesError(null);
    try {
      const res = await fetch(
        "https://api.github.com/repos/Lelequyngocquy/desktop-bubble-chat/releases/latest",
        {
          headers: {
            Accept: "application/vnd.github.v3+json",
          },
        },
      );

      if (!res.ok) {
        throw new Error(`GitHub API phản hồi mã ${res.status}`);
      }

      const data = await res.json();
      setGithubRelease({
        tag: data.tag_name || `v${availableVersion || currentVersion}`,
        name: data.name || "",
        publishedAt: data.published_at || "",
        htmlUrl:
          data.html_url ||
          "https://github.com/Lelequyngocquy/desktop-bubble-chat/releases",
        body: (data.body || "").trim(),
      });
    } catch (err) {
      console.warn(
        "[AppUpdateSection] Không thể tải release notes từ GitHub API:",
        err,
      );

      // Thử fallback sang latest.json do GitHub CDN lưu trữ
      try {
        const resLatest = await fetch(
          "https://github.com/Lelequyngocquy/desktop-bubble-chat/releases/latest/download/latest.json",
        );
        if (resLatest.ok) {
          const latestData = await resLatest.json();
          setGithubRelease({
            tag: `v${latestData.version}`,
            name: `DesktopBubbleChat v${latestData.version}`,
            publishedAt: latestData.pub_date || "",
            htmlUrl:
              "https://github.com/Lelequyngocquy/desktop-bubble-chat/releases/latest",
            body: (latestData.notes || "").trim(),
          });
          return;
        }
      } catch {
        // Fallback không thành công
      }

      // Nếu đã có releaseNotes từ tauri updater check
      if (releaseNotes) {
        setGithubRelease({
          tag: availableVersion ? `v${availableVersion}` : `v${currentVersion}`,
          name: "",
          publishedAt: releaseDate || "",
          htmlUrl:
            "https://github.com/Lelequyngocquy/desktop-bubble-chat/releases",
          body: releaseNotes.trim(),
        });
      } else {
        setNotesError(
          "Không thể kết nối đến GitHub để lấy thông tin phát hành.",
        );
      }
    } finally {
      setIsLoadingNotes(false);
    }
  }, [availableVersion, currentVersion, releaseDate, releaseNotes]);

  // Tự động tải release notes từ GitHub khi mở mục cài đặt hoặc khi phát hiện bản mới
  useEffect(() => {
    fetchGitHubReleaseNotes();
  }, [fetchGitHubReleaseNotes]);

  const handleOpenGitHubReleases = () => {
    systemService.openExternalUrl(
      githubRelease?.htmlUrl ||
        "https://github.com/Lelequyngocquy/desktop-bubble-chat/releases",
    );
  };

  return (
    <CommonLargeCard
      icon={<ArrowUpCircle className="w-4 h-4 text-primary" />}
      title="Cập nhật ứng dụng"
      subTitle={
        hasUpdate ? (
          <span className="px-2 py-0.5 rounded-pill text-xs font-semibold bg-primary text-white shadow-xs">
            Có bản mới v{availableVersion}
          </span>
        ) : (
          <span className="text-caption text-content-secondary">
            v{currentVersion}
          </span>
        )
      }
      linkTitle={
        <button
          onClick={handleOpenGitHubReleases}
          className="flex items-center gap-1.5 text-caption text-primary hover:underline font-medium transition-colors"
          title="Xem kho lưu trữ GitHub Releases"
        >
          <span>GitHub Releases</span>
          <ExternalLink className="w-3.5 h-3.5" />
        </button>
      }
    >
      <div className="flex flex-col gap-5">
        {/* KHỐI 1: Trạng thái tổng quan, Tiến trình tải & Các nút hành động chính */}
        <div className="p-4 bg-background-subtle/70 border border-border/80 rounded-panel flex flex-col gap-4">
          <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
            <div className="flex items-start gap-3.5">
              <div
                className={`w-12 h-12 rounded-xl flex items-center justify-center shrink-0 shadow-xs ${
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
                  <RefreshCw className="w-6 h-6 animate-spin" />
                ) : isAvailable ? (
                  <Sparkles className="w-6 h-6 animate-bounce" />
                ) : isDownloaded ? (
                  <Download className="w-6 h-6 text-white" />
                ) : isUpToDate ? (
                  <CheckCircle2 className="w-6 h-6" />
                ) : (
                  <ArrowUpCircle className="w-6 h-6" />
                )}
              </div>

              <div className="flex flex-col gap-0.5">
                <div className="flex items-center gap-2 flex-wrap">
                  <span className="text-body font-semibold text-content-primary">
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
                  </span>

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
                    ? "Đang kết nối đến GitHub CDN để quét dữ liệu..."
                    : isDownloading
                      ? `Tiến trình: ${progressPercent}% · ${formatBytes(downloadedBytes)} / ${formatBytes(totalBytes)} (${downloadSpeed})`
                      : isDownloaded
                        ? "Gói cài đặt đã được tải xuống và kiểm tra toàn vẹn. Nhấn 'Khởi động lại để cập nhật'."
                        : isAvailable
                          ? `Phiên bản v${availableVersion} đã sẵn sàng. Bạn có thể tải ngay bây giờ hoặc tiếp tục sử dụng.`
                          : isUpToDate
                            ? `DesktopBubbleChat v${currentVersion} là phiên bản ổn định nhất hiện hành.`
                            : "Nhấn nút bên dưới để kiểm tra các tính năng mới từ GitHub."}
                </p>

                {releaseDate && (
                  <span className="text-caption text-content-muted mt-0.5">
                    Ngày phát hành:{" "}
                    {new Date(releaseDate).toLocaleDateString("vi-VN")}
                  </span>
                )}
              </div>
            </div>

            {/* Các nút hành động chính */}
            <div className="flex items-center gap-2 flex-wrap shrink-0">
              {/* Nút Tải bản cập nhật */}
              {isAvailable && (
                <button
                  onClick={startDownload}
                  className="flex items-center gap-2 px-3.5 py-1.5 bg-primary hover:bg-primary-hover text-white rounded-control text-small font-semibold shadow-xs transition-colors cursor-pointer"
                >
                  <Download className="w-4 h-4" />
                  <span>Tải bản cập nhật</span>
                </button>
              )}

              {/* Nút Dừng tải */}
              {isDownloading && (
                <button
                  onClick={cancelDownload}
                  className="flex items-center gap-2 px-3.5 py-1.5 bg-rose-50 hover:bg-rose-100 text-rose-600 border border-rose-200 rounded-control text-small font-semibold transition-colors cursor-pointer"
                >
                  <Pause className="w-4 h-4" />
                  <span>Dừng tải</span>
                </button>
              )}

              {/* Nút Khởi động lại: CHỈ hiển thị khi đã tải xong thành công 100% */}
              {isDownloaded && (
                <button
                  onClick={installAndRelaunch}
                  disabled={isInstalling}
                  title="Khởi động lại ứng dụng để áp dụng bản cập nhật mới"
                  className="flex items-center gap-2 px-3.5 py-1.5 bg-emerald-600 hover:bg-emerald-700 text-white rounded-control text-small font-semibold shadow-xs transition-colors disabled:opacity-60 animate-pulse cursor-pointer"
                >
                  <RotateCcw
                    className={`w-4 h-4 ${isInstalling ? "animate-spin" : ""}`}
                  />
                  <span>
                    {isInstalling
                      ? "Đang áp dụng..."
                      : "Khởi động lại để cập nhật"}
                  </span>
                </button>
              )}

              {/* Nút Kiểm tra bản mới */}
              {!isDownloading && !isInstalling && (
                <button
                  onClick={() => checkForUpdates(false)}
                  disabled={isChecking}
                  className={`flex items-center gap-2 px-3 py-1.5 border border-border hover:bg-white bg-background-surface text-content-primary rounded-control text-small font-medium transition-colors cursor-pointer ${
                    isChecking ? "opacity-60 cursor-not-allowed" : ""
                  }`}
                >
                  <RefreshCw
                    className={`w-3.5 h-3.5 text-content-secondary ${
                      isChecking ? "animate-spin text-primary" : ""
                    }`}
                  />
                  <span>
                    {isChecking ? "Đang quét..." : "Kiểm tra bản mới"}
                  </span>
                </button>
              )}
            </div>
          </div>

          {/* Thanh tiến trình tải */}
          {(isDownloading || isDownloaded) && (
            <div className="flex flex-col gap-1.5 pt-2 border-t border-border/60">
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

              <div className="w-full h-2.5 bg-slate-100 rounded-pill overflow-hidden border border-border/40 p-0.5">
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

          {/* Banner thông báo lỗi nếu có */}
          {errorMessage && (
            <div className="p-3 bg-rose-50 border border-rose-200 rounded-control text-rose-700 text-small flex items-center justify-between gap-2">
              <div className="flex items-center gap-2">
                <Info className="w-4 h-4 shrink-0 text-rose-500" />
                <span>{errorMessage}</span>
              </div>
              <button
                onClick={() => checkForUpdates(false)}
                className="text-xs font-semibold text-rose-700 underline hover:no-underline shrink-0 cursor-pointer"
              >
                Thử lại
              </button>
            </div>
          )}

          {/* Hàng thông số & Toggle tự động */}
          <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 pt-2 border-t border-border/60 text-small">
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
        </div>

        {/* KHỐI 2: Nhật ký phát hành (Changelog / Release Notes) lấy thực tế từ GitHub */}
        <div className="flex flex-col gap-3">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <h3 className="text-body font-semibold text-content-primary m-0">
                {githubRelease?.tag
                  ? `Điểm mới trong bản phát hành ${githubRelease.tag}`
                  : availableVersion
                    ? `Điểm mới trong bản phát hành v${availableVersion}`
                    : `Thông tin phiên bản v${currentVersion}`}
              </h3>
            </div>

            <button
              onClick={fetchGitHubReleaseNotes}
              disabled={isLoadingNotes}
              className="flex items-center gap-1.5 text-caption text-content-secondary hover:text-primary transition-colors cursor-pointer"
              title="Làm mới nội dung ghi chú từ GitHub"
            >
              <RefreshCw
                className={`w-3.5 h-3.5 ${isLoadingNotes ? "animate-spin text-primary" : ""}`}
              />
              <span>
                {isLoadingNotes ? "Đang đồng bộ..." : "Làm mới ghi chú"}
              </span>
            </button>
          </div>

          <div className="bg-background-app/70 border border-border/80 rounded-control p-4 text-small text-content-primary leading-relaxed whitespace-pre-wrap font-sans min-h-[90px]">
            {isLoadingNotes ? (
              <div className="flex items-center justify-center gap-2.5 py-6 text-content-secondary">
                <RefreshCw className="w-4 h-4 animate-spin text-primary" />
                <span>
                  Đang tải thông tin bản phát hành thực tế từ GitHub...
                </span>
              </div>
            ) : notesError ? (
              <div className="flex flex-col items-center justify-center gap-2 py-4 text-content-muted">
                <span>{notesError}</span>
                <button
                  onClick={fetchGitHubReleaseNotes}
                  className="text-xs text-primary underline hover:no-underline font-medium cursor-pointer"
                >
                  Thử tải lại
                </button>
              </div>
            ) : githubRelease?.body ? (
              githubRelease.body
            ) : githubRelease ? (
              <div className="text-content-secondary">
                <p className="m-0">
                  Bản phát hành{" "}
                  <strong className="text-content-primary">
                    {githubRelease.tag}
                  </strong>
                  {githubRelease.name ? ` (${githubRelease.name})` : ""} đã sẵn
                  sàng trên GitHub
                  {githubRelease.publishedAt
                    ? ` (Ngày đăng: ${new Date(githubRelease.publishedAt).toLocaleDateString("vi-VN")})`
                    : ""}
                  .
                </p>
              </div>
            ) : (
              <div className="text-content-muted text-center py-4">
                Chưa có dữ liệu phát hành. Nhấn &apos;Làm mới ghi chú&apos; để
                tải từ GitHub.
              </div>
            )}
          </div>
        </div>
      </div>
    </CommonLargeCard>
  );
};

export default AppUpdateSection;
