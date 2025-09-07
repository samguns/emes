<template>
  <div class="about">
    <h1>This is an about page</h1>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue';
import sockNsAiService from '@/services/sockNsAi';

onMounted(() => {
  // sockNsAiService.on('message', (data: any) => {
  //   console.log(data);
  // });

  if (sockNsAiService.isConnected()) {
    // sockNsAiService.sendMessage('message', 'Hello, server from the other side!');
    sockNsAiService.sendMessage('training:ack', 'what');
    return;
  }

  sockNsAiService.on('connected', () => {
    console.log('Connected to AI namespace');
    sockNsAiService.sendMessage('message', 'Hello, server!');
  });

  sockNsAiService.on('message', (data: any) => {
    console.log(data);
  });
});
</script>

<style>
@media (min-width: 1024px) {
  .about {
    min-height: 100vh;
    display: flex;
    align-items: center;
  }
}
</style>
