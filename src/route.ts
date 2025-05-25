import { createRouter, createMemoryHistory } from "vue-router";
import Login from "./components/Login.vue";

export const router = createRouter({
  history: createMemoryHistory(),
  routes: [
    {
      path: "/",
      component: Login
    },
  ],
})
