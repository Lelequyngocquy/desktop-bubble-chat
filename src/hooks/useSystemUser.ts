import { useEffect, useCallback } from "react";
import { useSystemUserStore } from "../stores/systemUserStore";
import { systemService } from "../services/systemService";

export function useSystemUser() {
  const systemUser = useSystemUserStore((s) => s.systemUser);
  const systemUserInitials = useSystemUserStore((s) => s.systemUserInitials);
  const setSystemUser = useSystemUserStore((s) => s.setSystemUser);

  const fetchSystemUser = useCallback(async () => {
    const name = await systemService.getSystemUsername();
    setSystemUser(name);
  }, [setSystemUser]);

  useEffect(() => {
    fetchSystemUser();
  }, [fetchSystemUser]);

  return {
    systemUser,
    systemUserInitials,
    fetchSystemUser,
  };
}
