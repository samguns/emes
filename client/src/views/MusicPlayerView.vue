<template>
    <div class="music-player-view">
        <MusicPlayer />
    </div>
</template>

<script>
import MusicPlayer from '@/components/MusicPlayer.vue'

export default {
    name: 'MusicPlayerView',
    components: {
        MusicPlayer
    }
}
</script>

<style scoped>
.music-player-view {
    padding: 20px;
    max-width: 800px;
    margin: 0 auto;
}
</style>