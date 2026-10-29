import { useEffect, useCallback } from "react";
import { useSystemUserStore } from "../stores/systemUserStore";
import { systemService } from "../services/systemService";

export function useSystemUser() {
  const { systemUser, systemUserInitials, setSystemUser } = useSystemUserStore();

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
