import { useCallback } from "react";
import { useSystemUserStore } from "../stores/systemUserStore";
import { systemService } from "../services/systemService";

export function useSystemUser() {
  const systemUser = useSystemUserStore((s) => s.systemUser);
  const systemUserInitials = useSystemUserStore((s) => s.systemUserInitials);
  const fetchSystemUser = useCallback(async () => {
    const name = await systemService.getSystemUsername();
    useSystemUserStore.getState().setSystemUser(name);
  }, []);
  return {
    systemUser,
    systemUserInitials,
    fetchSystemUser,
  };
}
