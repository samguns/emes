<script setup lang="ts">
import { RouterLink, RouterView } from 'vue-router'
</script>

<template>
  <div class="app-container">
    <nav class="mobile-nav">
      <RouterLink to="/" class="nav-item">
        <span class="nav-icon">📊</span>
        <span class="nav-text">首页</span>
      </RouterLink>
      <RouterLink to="/about" class="nav-item">
        <span class="nav-icon">ℹ️</span>
        <span class="nav-text">关于</span>
      </RouterLink>
    </nav>

    <main class="content">
      <RouterView />
    </main>
  </div>
</template>

<style scoped>
.app-container {
  display: flex;
  flex-direction: column;
  height: 100vh;
  width: 100%;
  overflow-y: auto;
  background: linear-gradient(135deg, #66ead8 0%, #04ff3f 100%);
}

.content {
  flex: 1;
  overflow-y: auto;
  position: relative;
}

.mobile-nav {
  display: flex;
  justify-content: space-around;
  background-color: var(--color-background-soft);
  border-top: 1px solid var(--color-border);
  padding: 0.5rem 0;
  position: fixed;
  bottom: 0;
  width: 100%;
  z-index: 10;
}

.nav-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 0.5rem;
  text-decoration: none;
  color: var(--color-text);
  width: 100%;
  text-align: center;
}

.nav-icon {
  font-size: 1.5rem;
  margin-bottom: 0.25rem;
}

.nav-text {
  font-size: 0.75rem;
}

.nav-item.router-link-active {
  color: var(--color-primary);
}

/* Desktop styles */
@media (min-width: 768px) {
  .app-container {
    flex-direction: row;
  }

  .mobile-nav {
    flex-direction: column;
    width: 200px;
    height: 100%;
    position: static;
    border-top: none;
    border-right: 1px solid var(--color-border);
    padding: 1rem 0;
  }

  .nav-item {
    flex-direction: row;
    justify-content: flex-start;
    padding: 0.75rem 1rem;
    gap: 0.5rem;
  }

  .nav-icon {
    margin-bottom: 0;
  }

  .nav-text {
    font-size: 0.9rem;
  }

  .content {
    flex: 1;
  }
}
</style>
