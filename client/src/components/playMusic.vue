<template>
  <div class="container">
    <h1>🎵 音乐歌单</h1>
    <div class="playlist-grid">
      <div class="playlist-card sports" @click="goToSports()">
        <span class="playlist-icon">🏃‍♂️</span>
        <div class="playlist-title">运动歌单</div>
        <div class="playlist-desc">充满活力的音乐，让你运动更有动力</div>
      </div>
      
      <div class="playlist-card study" @click="goToStudy()">
        <span class="playlist-icon">📚</span>
        <div class="playlist-title">学习歌单</div>
        <div class="playlist-desc">专注学习的背景音乐，提高学习效率</div>
      </div>
      
      <div class="playlist-card sleep" @click="goToSleep()">
        <span class="playlist-icon">😴</span>
        <div class="playlist-title">睡前歌单</div>
        <div class="playlist-desc">舒缓的音乐，助你安然入睡</div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
function goToSports() {
  const sportsPage = window.open('', '_blank');
  sportsPage?.document.write(`
    <!DOCTYPE html>
    <html lang="zh-CN">
    <head>
      <meta charset="UTF-8">
      <meta name="viewport" content="width=device-width, initial-scale=1.0">
      <title>运动歌单</title>
      <style>
        body {
          margin: 0;
          padding: 0;
          background: linear-gradient(135deg, #f30703 0%, #f15d80 100%);
          font-family: 'Microsoft YaHei', Arial, sans-serif;
          display: flex;
          justify-content: center;
          align-items: center;
          min-height: 100vh;
        }
        .sports-container {
          text-align: center;
          color: white;
          background: rgba(255, 255, 255, 0.1);
          padding: 3rem;
          border-radius: 20px;
          backdrop-filter: blur(10px);
          max-width: 500px;
          width: 90%;
        }
        .sports-icon {
          font-size: 4rem;
          margin-bottom: 1rem;
        }
        .sports-title {
          font-size: 2rem;
        }
      </style>
    </head>
    <body>
      <div class="sports-container">
        <div class="sports-icon">🏃‍♂️</div>
        <h1 class="sports-title">运动歌单</h1>
        <p>正在播放运动音乐...</p>
      </div>
    </body>
    </html>
  `);
}

function goToStudy() {
  const studyPage = window.open('', '_blank');
  studyPage?.document.write(`
    <!DOCTYPE html>
    <html lang="zh-CN">
    <head>
      <meta charset="UTF-8">
      <meta name="viewport" content="width=device-width, initial-scale=1.0">
      <title>学习歌单</title>
      <style>
        body {
          margin: 0;
          padding: 0;
          background: linear-gradient(135deg, #4ecdc4 0%, #6dd5ed 100%);
          font-family: 'Microsoft YaHei', Arial, sans-serif;
          display: flex;
          justify-content: center;
          align-items: center;
          min-height: 100vh;
        }
        .study-container {
          text-align: center;
          color: white;
          background: rgba(255, 255, 255, 0.1);
          padding: 3rem;
          border-radius: 20px;
          backdrop-filter: blur(10px);
          max-width: 500px;
          width: 90%;
        }
        .study-icon {
          font-size: 4rem;
          margin-bottom: 1rem;
        }
        .study-title {
          font-size: 2rem;
        }
      </style>
    </head>
    <body>
      <div class="study-container">
        <div class="study-icon">📚</div>
        <h1 class="study-title">学习歌单</h1>
        <p>正在播放学习音乐...</p>
      </div>
    </body>
    </html>
  `);
}

function goToSleep() {
  const sleepPage = window.open('', '_blank');
  sleepPage?.document.write(`
    <!DOCTYPE html>
    <html lang="zh-CN">
    <head>
      <meta charset="UTF-8">
      <meta name="viewport" content="width=device-width, initial-scale=1.0">
      <title>睡前歌单</title>
      <style>
        body {
          margin: 0;
          padding: 0;
          background: linear-gradient(135deg, #a8edea 0%, #fed6e3 100%);
          font-family: 'Microsoft YaHei', Arial, sans-serif;
          display: flex;
          justify-content: center;
          align-items: center;
          min-height: 100vh;
        }
        .sleep-container {
          text-align: center;
          color: #333;
          background: rgba(255, 255, 255, 0.7);
          padding: 3rem;
          border-radius: 20px;
          backdrop-filter: blur(10px);
          max-width: 500px;
          width: 90%;
        }
        .sleep-icon {
          font-size: 4rem;
          margin-bottom: 1rem;
        }
        .sleep-title {
          font-size: 2rem;
        }
      </style>
    </head>
    <body>
      <div class="sleep-container">
        <div class="sleep-icon">😴</div>
        <h1 class="sleep-title">睡前歌单</h1>
        <p>正在播放舒缓音乐...</p>
      </div>
    </body>
    </html>
  `);
}
</script>

<style scoped>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

.container {
  text-align: center;
  background: rgba(255, 255, 255, 0.95);
  padding: 3rem;
  border-radius: 20px;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.1);
  backdrop-filter: blur(10px);
  max-width: 600px;
  width: 90%;
  margin: 2rem auto;
}

h1 {
  color: #333;
  margin-bottom: 2rem;
  font-size: 2.5rem;
  font-weight: 300;
}

.playlist-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 1.5rem;
  margin-top: 2rem;
}

.playlist-card {
  background: linear-gradient(145deg, #ffffff, #f0f0f0);
  border-radius: 15px;
  padding: 2rem;
  cursor: pointer;
  transition: all 0.3s ease;
  border: 2px solid transparent;
  position: relative;
  overflow: hidden;
}

.playlist-card:hover {
  transform: translateY(-10px);
  box-shadow: 0 15px 30px rgba(0, 0, 0, 0.2);
  border-color: #667eea;
}

.playlist-card::before {
  content: '';
  position: absolute;
  top: 0;
  left: -100%;
  width: 100%;
  height: 100%;
  background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.4), transparent);
  transition: left 0.5s;
}

.playlist-card:hover::before {
  left: 100%;
}

.playlist-icon {
  font-size: 3rem;
  margin-bottom: 1rem;
  display: block;
}

.playlist-title {
  font-size: 1.5rem;
  font-weight: 600;
  color: #333;
  margin-bottom: 0.5rem;
}

.playlist-desc {
  color: #666;
  font-size: 0.9rem;
  line-height: 1.4;
}

/* 不同歌单的颜色主题 */
.sports {
  background: linear-gradient(145deg, #f30703, #f15d80);
}

.study {
  background: linear-gradient(145deg, #4ecdc4, #6dd5ed);
}

.sleep {
  background: linear-gradient(145deg, #a8edea, #fed6e3);
}

.sports .playlist-title,
.study .playlist-title,
.sleep .playlist-title {
  color: white;
}

.sports .playlist-desc,
.study .playlist-desc,
.sleep .playlist-desc {
  color: rgba(255, 255, 255, 0.8);
}

@media (max-width: 768px) {
  .container {
    padding: 2rem;
    margin: 1rem;
  }
  
  h1 {
    font-size: 2rem;
  }
  
  .playlist-grid {
    grid-template-columns: 1fr;
  }
}
</style>