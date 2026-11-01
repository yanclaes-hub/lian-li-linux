import { createRouter, createWebHashHistory, type RouteRecordRaw } from "vue-router";

const routes: RouteRecordRaw[] = [
  { path: "/", redirect: "/devices" },
  { path: "/devices", name: "devices", component: () => import("@/views/DevicesPage.vue") },
  { path: "/fans", name: "fans", component: () => import("@/views/FansPage.vue") },
  { path: "/rgb", name: "rgb", component: () => import("@/views/RgbPage.vue") },
  { path: "/settings", name: "settings", component: () => import("@/views/SettingsPage.vue") },
  { path: "/installation", name: "installation", component: () => import("@/views/InstallationPage.vue") },
];

export const router = createRouter({
  history: createWebHashHistory(),
  routes,
});

export const MAIN_ROUTES = [
  { name: "devices", labelKey: "nav.devices", icon: "monitor", to: "/devices" },
  { name: "fans", labelKey: "nav.fans", icon: "fan", to: "/fans" },
  { name: "rgb", labelKey: "nav.rgb", icon: "palette", to: "/rgb" },
  { name: "settings", labelKey: "nav.settings", icon: "settings", to: "/settings" },
  { name: "installation", labelKey: "nav.installation", icon: "health", to: "/installation" },
] as const;
