#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
音频可视化与WS2812 LED控制
分析MP3音频文件，并将波形数据转换为WS2812 LED灯光效果
"""

import numpy as np
import os
import sys
import subprocess
import time
import random
from datetime import datetime
import argparse
import warnings
import json

# 导入WS2812控制模块
from ws2812 import SPIws2812

# 忽略numpy相关的警告
warnings.filterwarnings('ignore', category=FutureWarning)
warnings.filterwarnings('ignore', category=UserWarning)

try:
    import librosa
    LIBROSA_AVAILABLE = True
    print("✅ Librosa加载成功")
except ImportError as e:
    print(f"⚠️ Librosa加载失败: {e}")
    print("将使用基础音频处理方法")
    LIBROSA_AVAILABLE = False

class AudioProcessor:
    """音频处理类"""
    
    def __init__(self, audio_path):
        self.audio_path = audio_path
        self.audio_data = None
        self.sample_rate = 44100
        self.duration = 0
        
    def load_audio_basic(self):
        """使用基础方法加载音频（通过ffmpeg）"""
        try:
            # 使用ffmpeg提取音频数据
            cmd = [
                'ffmpeg', '-i', self.audio_path, 
                '-f', 'f64le', '-acodec', 'pcm_f64le', 
                '-ar', str(self.sample_rate), '-ac', '1', '-'
            ]
            
            result = subprocess.run(cmd, capture_output=True, check=True)
            
            # 将字节数据转换为numpy数组
            audio_data = np.frombuffer(result.stdout, dtype=np.float64)
            self.audio_data = audio_data.astype(np.float32)  # 转换为float32节省内存
            self.duration = len(self.audio_data) / self.sample_rate
            
            print(f"✅ 音频加载成功 (基础方法)")
            print(f"   时长: {self.duration:.2f} 秒")
            print(f"   采样率: {self.sample_rate} Hz")
            print(f"   样本数: {len(self.audio_data)}")
            
            return True
            
        except subprocess.CalledProcessError as e:
            print(f"❌ FFmpeg处理失败: {e}")
            return False
        except Exception as e:
            print(f"❌ 音频加载失败: {e}")
            return False
    
    def load_audio_librosa(self):
        """使用librosa加载音频"""
        try:
            self.audio_data, self.sample_rate = librosa.load(self.audio_path, sr=None)
            self.duration = len(self.audio_data) / self.sample_rate
            
            print(f"✅ 音频加载成功 (Librosa)")
            print(f"   时长: {self.duration:.2f} 秒")
            print(f"   采样率: {self.sample_rate} Hz")
            
            return True
        except Exception as e:
            print(f"❌ Librosa加载失败: {e}")
            return False
    
    def load_audio(self):
        """加载音频文件"""
        if LIBROSA_AVAILABLE:
            if self.load_audio_librosa():
                return True
        
        return self.load_audio_basic()
    
    def compute_waveform(self, window_size=1024, hop_length=512):
        """计算波形数据"""
        if self.audio_data is None:
            return None
            
        # 计算每秒的帧数
        frames_per_second = self.sample_rate / hop_length
        total_frames = int(self.duration * frames_per_second)
        
        # 存储波形数据，每60ms一次
        waveform_data = []
        
        # 计算60ms对应的帧数
        frames_per_60ms = int(frames_per_second * 0.06)  # 60ms的帧数
        
        for i in range(total_frames):
            # 计算当前时间窗口
            start_sample = int(i * hop_length)
            end_sample = min(start_sample + window_size, len(self.audio_data))
            
            if start_sample >= len(self.audio_data):
                break
                
            # 获取当前窗口的音频数据
            audio_window = self.audio_data[start_sample:end_sample]
            if len(audio_window) < window_size:
                audio_window = np.pad(audio_window, (0, window_size - len(audio_window)))
            
            # 计算波形振幅（取绝对值的平均）
            amplitudes = np.abs(audio_window)
            
            # 每60ms保存一次数据
            if i % frames_per_60ms == 0:
                # 计算整体振幅
                overall_amplitude = np.mean(amplitudes)
                
                # 将波形数据重新采样为16个点
                resampled = []
                chunk_size = len(amplitudes) // 16
                for j in range(16):
                    start_idx = j * chunk_size
                    end_idx = (j + 1) * chunk_size
                    if end_idx > start_idx:
                        chunk_avg = np.mean(amplitudes[start_idx:end_idx])
                        resampled.append(chunk_avg)
                    else:
                        resampled.append(0)
                
                # 归一化到0-1范围
                max_val = max(resampled) if max(resampled) > 0 else 1
                normalized = [amp / max_val for amp in resampled]
                
                waveform_data.append({
                    'normalized': normalized,
                    'overall_amplitude': overall_amplitude
                })
        
        return waveform_data

    def waveform_to_pixels(self, waveform_data):
        """将波形数据转换为LED像素数据"""
        pixels_data = []
        
        for frame in waveform_data:
            # 获取归一化的振幅数据和整体振幅
            normalized = frame['normalized']
            overall_amplitude = frame['overall_amplitude']
            
            # 根据整体振幅确定要点亮的LED数量
            # 归一化整体振幅
            max_amplitude = np.max([f['overall_amplitude'] for f in waveform_data]) if waveform_data else 1
            relative_amplitude = overall_amplitude / max_amplitude if max_amplitude > 0 else 0
            
            # 计算要点亮的LED数量，范围1-16
            active_leds = max(1, min(16, int(relative_amplitude * 16) + 1))
            
            # 为每一帧创建一组像素
            frame_pixels = []
            
            # 生成一个随机颜色，用于整个帧
            base_r = random.randint(50, 255)
            base_g = random.randint(50, 255)
            base_b = random.randint(50, 255)
            
            # 为每个LED设置颜色
            for i in range(16):
                if i < active_leds:
                    # 点亮的LED，亮度随索引递减
                    brightness_factor = 1.0 - (i / active_leds * 0.7)  # 亮度从100%递减到30%
                    r = int(base_r * brightness_factor)
                    g = int(base_g * brightness_factor)
                    b = int(base_b * brightness_factor)
                    frame_pixels.append([r, g, b])
                else:
                    # 未点亮的LED
                    frame_pixels.append([0, 0, 0])
            
            pixels_data.append(frame_pixels)
        
        return pixels_data

    def save_pixels_data(self, pixels_data, output_file):
        """将像素数据保存到JSON文件"""
        with open(output_file, 'w') as f:
            json.dump(pixels_data, f)
        print(f"✅ 像素数据已保存到 {output_file}")

class LEDController:
    """WS2812 LED控制类"""
    
    def __init__(self, num_leds=16):
        self.num_leds = num_leds
        self.spi = None
        
    def initialize(self):
        """初始化SPI和WS2812"""
        try:
            self.spi = SPIws2812.init((0, 0), self.num_leds)
            print("✅ WS2812 LED初始化成功")
            return True
        except Exception as e:
            print(f"❌ WS2812 LED初始化失败: {e}")
            return False
    
    def display_frame(self, pixels):
        """显示单帧像素数据"""
        if self.spi is None:
            return False
        
        try:
            self.spi.write(pixels)
            return True
        except Exception as e:
            print(f"❌ LED显示失败: {e}")
            return False
    
    def clear(self):
        """清除所有LED"""
        if self.spi is None:
            return
            
        empty_pixels = [[0, 0, 0] for _ in range(self.num_leds)]
        self.spi.write(empty_pixels)

def play_audio(audio_file):
    """播放音频文件"""
    try:
        # 使用ffplay播放音频
        cmd = ['ffplay', '-nodisp', '-autoexit', audio_file]
        return subprocess.Popen(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    except Exception as e:
        print(f"❌ 音频播放失败: {e}")
        return None

def main():
    parser = argparse.ArgumentParser(description='音频可视化与WS2812 LED控制')
    parser.add_argument('audio_file', help='音频文件路径')
    parser.add_argument('--analyze-only', action='store_true', help='仅分析音频，不播放')
    parser.add_argument('--output', default='pixels_data.json', help='像素数据输出文件')
    parser.add_argument('--num-leds', type=int, default=16, help='LED数量')
    
    args = parser.parse_args()
    
    # 检查文件是否存在
    if not os.path.exists(args.audio_file):
        print(f"❌ 文件不存在: {args.audio_file}")
        return
    
    # 初始化音频处理器
    audio_processor = AudioProcessor(args.audio_file)
    if not audio_processor.load_audio():
        return
    
    print("🔍 分析音频波形...")
    waveform_data = audio_processor.compute_waveform()
    if waveform_data is None:
        print("❌ 波形分析失败")
        return
    
    print(f"✅ 波形分析完成，共 {len(waveform_data)} 帧数据")
    
    # 将波形数据转换为像素数据
    print("🎨 生成像素数据...")
    pixels_data = audio_processor.waveform_to_pixels(waveform_data)
    
    # 保存像素数据
    audio_processor.save_pixels_data(pixels_data, args.output)
    
    if args.analyze_only:
        print("✅ 分析完成，未播放音频")
        return
    
    # 初始化LED控制器
    led_controller = LEDController(args.num_leds)
    if not led_controller.initialize():
        return
    
    print("🎵 开始播放音频和LED灯光...")
    
    # 播放音频
    audio_process = play_audio(args.audio_file)
    if audio_process is None:
        led_controller.clear()
        return
    
            # 显示LED灯效
    try:
        frame_duration = 0.06  # 每60ms更新一次
        for frame in pixels_data:
            led_controller.display_frame(frame)
            time.sleep(frame_duration)
            
            # 检查音频是否仍在播放
            if audio_process.poll() is not None:
                break
    except KeyboardInterrupt:
        print("\n⏹️ 用户中断")
    finally:
        # 清理资源
        if audio_process and audio_process.poll() is None:
            audio_process.terminate()
        led_controller.clear()
    
    print("✅ 播放完成")

if __name__ == "__main__":
    main()
