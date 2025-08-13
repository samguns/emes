import { createRouter, createWebHistory } from 'vue-router'
import HomeView from '../views/HomeView.vue'
import FileUploaderView from '../views/FileUploaderView.vue'
import PlayMusicView from '../views/PlayMusicView.vue'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'home',
      component: HomeView,
    },
    {
      path: '/about',
      name: 'about',
      // route level code-splitting
      // this generates a separate chunk (About.[hash].js) for this route
      // which is lazy-loaded when the route is visited.
      component: () => import('../views/AboutView.vue'),
    },
    {
      path: '/upload',
      name: 'upload',
      // route for file uploader
      component: FileUploaderView,
    },
    {
      path: '/music-player',
      name: 'music-player',
      // route for music player
      component: PlayMusicView,
    }
  ],
})

export default router
