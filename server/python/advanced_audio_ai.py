import librosa
import librosa.display
import matplotlib.pyplot as plt
import numpy as np
import pandas as pd
import seaborn as sns
import os  # 添加os模块导入
from sklearn.ensemble import RandomForestClassifier, GradientBoostingClassifier
from sklearn.neural_network import MLPClassifier
from sklearn.model_selection import train_test_split, cross_val_score
from sklearn.preprocessing import StandardScaler, LabelEncoder
from sklearn.metrics import classification_report, confusion_matrix, accuracy_score
from sklearn.decomposition import PCA
from sklearn.cluster import KMeans
import tensorflow as tf
from tensorflow.keras.models import Sequential
from tensorflow.keras.layers import Dense, Dropout, LSTM, Conv1D, MaxPooling1D
from tensorflow.keras.utils import to_categorical
import warnings
warnings.filterwarnings('ignore')

class AdvancedAudioAI:
    def __init__(self):
        self.scaler = StandardScaler()
        self.pca = PCA(n_components=20)
        self.label_encoder = LabelEncoder()
        self.models = {
            'random_forest': RandomForestClassifier(n_estimators=200, random_state=42),
            'gradient_boosting': GradientBoostingClassifier(n_estimators=100, random_state=42),
            'neural_network': MLPClassifier(hidden_layer_sizes=(100, 50), max_iter=500, random_state=42)
        }
        self.deep_model = None
        self.feature_names = []
        
    def extract_advanced_features(self, audio_path):
        """提取高级音频特征"""
        print(f"🔍 正在提取高级特征: {audio_path}")
        
        audio, sr = librosa.load(audio_path)
        
        features = {}
        
        # 1. 基础特征
        features['duration'] = librosa.get_duration(y=audio, sr=sr)
        features['sample_rate'] = sr
        features['n_samples'] = len(audio)
        
        # 2. 频谱特征
        # 梅尔频谱图
        mel_spec = librosa.feature.melspectrogram(y=audio, sr=sr, n_mels=128)
        features['mel_spectrogram_mean'] = np.mean(mel_spec)
        features['mel_spectrogram_std'] = np.std(mel_spec)
        features['mel_spectrogram_max'] = np.max(mel_spec)
        features['mel_spectrogram_min'] = np.min(mel_spec)
        
        # 3. MFCC特征 (更多系数)
        mfccs = librosa.feature.mfcc(y=audio, sr=sr, n_mfcc=20)
        for i in range(20):
            features[f'mfcc_{i}_mean'] = np.mean(mfccs[i])
            features[f'mfcc_{i}_std'] = np.std(mfccs[i])
            features[f'mfcc_{i}_max'] = np.max(mfccs[i])
            features[f'mfcc_{i}_min'] = np.min(mfccs[i])
        
        # 4. 节奏和节拍特征
        tempo, beats = librosa.beat.beat_track(y=audio, sr=sr)
        features['tempo'] = tempo
        features['beat_count'] = len(beats)
        features['beat_interval_mean'] = np.mean(np.diff(beats)) if len(beats) > 1 else 0
        
        # 5. 音高特征
        pitches, magnitudes = librosa.piptrack(y=audio, sr=sr)
        features['pitch_mean'] = np.mean(pitches)
        features['pitch_std'] = np.std(pitches)
        features['pitch_max'] = np.max(pitches)
        features['pitch_min'] = np.min(pitches)
        
        # 6. 能量特征
        rms = librosa.feature.rms(y=audio)
        features['rms_mean'] = np.mean(rms)
        features['rms_std'] = np.std(rms)
        features['rms_max'] = np.max(rms)
        features['rms_min'] = np.min(rms)
        
        # 7. 零交叉率
        zcr = librosa.feature.zero_crossing_rate(audio)
        features['zcr_mean'] = np.mean(zcr)
        features['zcr_std'] = np.std(zcr)
        features['zcr_max'] = np.max(zcr)
        
        # 8. 频谱特征
        spectral_centroids = librosa.feature.spectral_centroid(y=audio, sr=sr)
        features['spectral_centroid_mean'] = np.mean(spectral_centroids)
        features['spectral_centroid_std'] = np.std(spectral_centroids)
        features['spectral_centroid_max'] = np.max(spectral_centroids)
        
        spectral_bandwidth = librosa.feature.spectral_bandwidth(y=audio, sr=sr)
        features['spectral_bandwidth_mean'] = np.mean(spectral_bandwidth)
        features['spectral_bandwidth_std'] = np.std(spectral_bandwidth)
        
        spectral_rolloff = librosa.feature.spectral_rolloff(y=audio, sr=sr)
        features['spectral_rolloff_mean'] = np.mean(spectral_rolloff)
        features['spectral_rolloff_std'] = np.std(spectral_rolloff)
        
        # 9. 谐波和打击乐分离
        harmonic, percussive = librosa.effects.hpss(audio)
        features['harmonic_ratio'] = np.sum(harmonic**2) / (np.sum(harmonic**2) + np.sum(percussive**2))
        features['percussive_ratio'] = 1 - features['harmonic_ratio']
        
        # 10. 色度特征
        chroma = librosa.feature.chroma_stft(y=audio, sr=sr)
        features['chroma_mean'] = np.mean(chroma)
        features['chroma_std'] = np.std(chroma)
        
        # 11. 音调特征
        tonnetz = librosa.feature.tonnetz(y=harmonic, sr=sr)
        features['tonnetz_mean'] = np.mean(tonnetz)
        features['tonnetz_std'] = np.std(tonnetz)
        
        # 12. 频谱对比度
        contrast = librosa.feature.spectral_contrast(y=audio, sr=sr)
        features['spectral_contrast_mean'] = np.mean(contrast)
        features['spectral_contrast_std'] = np.std(contrast)
        
        # 13. 多普勒特征
        poly_features = librosa.feature.poly_features(y=audio, sr=sr)
        features['poly_features_mean'] = np.mean(poly_features)
        features['poly_features_std'] = np.std(poly_features)
        
        return features
    
    def create_deep_learning_model(self, input_shape, num_classes):
        """创建深度学习模型"""
        model = Sequential([
            # 卷积层
            Conv1D(64, 3, activation='relu', input_shape=input_shape),
            MaxPooling1D(2),
            Conv1D(128, 3, activation='relu'),
            MaxPooling1D(2),
            Conv1D(256, 3, activation='relu'),
            MaxPooling1D(2),
            
            # LSTM层
            LSTM(128, return_sequences=True),
            Dropout(0.3),
            LSTM(64),
            Dropout(0.3),
            
            # 全连接层
            Dense(128, activation='relu'),
            Dropout(0.3),
            Dense(64, activation='relu'),
            Dense(num_classes, activation='softmax')
        ])
        
        model.compile(
            optimizer='adam',
            loss='categorical_crossentropy',
            metrics=['accuracy']
        )
        
        return model
    
    def prepare_deep_learning_data(self, audio_paths, labels):
        """准备深度学习数据"""
        print("🔄 准备深度学习数据...")
        
        X_deep = []
        y_deep = []
        
        for audio_path in audio_paths:
            try:
                audio, sr = librosa.load(audio_path, sr=22050)
                
                # 提取MFCC特征作为深度学习输入
                mfccs = librosa.feature.mfcc(y=audio, sr=sr, n_mfcc=13)
                
                # 确保所有序列长度一致
                if mfccs.shape[1] > 1000:
                    mfccs = mfccs[:, :1000]
                elif mfccs.shape[1] < 1000:
                    # 填充到1000
                    padding = np.zeros((13, 1000 - mfccs.shape[1]))
                    mfccs = np.hstack([mfccs, padding])
                
                X_deep.append(mfccs.T)  # 转置以匹配Conv1D输入格式
                
            except Exception as e:
                print(f"⚠️ 跳过文件 {audio_path}: {e}")
                continue
        
        # 编码标签
        y_encoded = self.label_encoder.fit_transform(labels[:len(X_deep)])
        y_categorical = to_categorical(y_encoded)
        
        return np.array(X_deep), y_categorical
    
    def train_models(self, audio_paths, labels):
        """训练多个AI模型"""
        print("🤖 开始训练AI模型...")
        
        # 提取特征
        features_list = []
        valid_labels = []
        
        for i, audio_path in enumerate(audio_paths):
            try:
                features = self.extract_advanced_features(audio_path)
                features_list.append(features)
                valid_labels.append(labels[i])
            except Exception as e:
                print(f"⚠️ 跳过文件 {audio_path}: {e}")
                continue
        
        if len(features_list) < 2:
            print("❌ 有效数据不足，无法训练模型")
            return
        
        # 转换为DataFrame
        df = pd.DataFrame(features_list)
        self.feature_names = df.columns.tolist()
        
        # 准备数据
        X = df.values
        y = np.array(valid_labels)
        
        # 编码标签
        y_encoded = self.label_encoder.fit_transform(y)
        
        # 分割数据
        X_train, X_test, y_train, y_test = train_test_split(
            X, y_encoded, test_size=0.2, random_state=42, stratify=y_encoded
        )
        
        # 标准化
        X_train_scaled = self.scaler.fit_transform(X_train)
        X_test_scaled = self.scaler.transform(X_test)
        
        # 训练传统机器学习模型
        results = {}
        for name, model in self.models.items():
            print(f"🔄 训练 {name}...")
            model.fit(X_train_scaled, y_train)
            
            # 评估
            y_pred = model.predict(X_test_scaled)
            accuracy = accuracy_score(y_test, y_pred)
            results[name] = {
                'accuracy': accuracy,
                'predictions': y_pred,
                'model': model
            }
            print(f"   {name} 准确率: {accuracy:.3f}")
        
        # 训练深度学习模型
        print("🔄 训练深度学习模型...")
        X_deep, y_deep = self.prepare_deep_learning_data(audio_paths, labels)
        
        if len(X_deep) > 0:
            X_deep_train, X_deep_test, y_deep_train, y_deep_test = train_test_split(
                X_deep, y_deep, test_size=0.2, random_state=42
            )
            
            self.deep_model = self.create_deep_learning_model(
                input_shape=(X_deep.shape[1], X_deep.shape[2]),
                num_classes=y_deep.shape[1]
            )
            
            history = self.deep_model.fit(
                X_deep_train, y_deep_train,
                epochs=50,
                batch_size=32,
                validation_data=(X_deep_test, y_deep_test),
                verbose=1
            )
            
            # 评估深度学习模型
            deep_accuracy = self.deep_model.evaluate(X_deep_test, y_deep_test)[1]
            results['deep_learning'] = {
                'accuracy': deep_accuracy,
                'model': self.deep_model,
                'history': history
            }
            print(f"   深度学习模型准确率: {deep_accuracy:.3f}")
        
        return results, X_test, y_test
    
    def analyze_audio_cluster(self, audio_paths):
        """使用聚类分析音频特征"""
        print("🔍 执行聚类分析...")
        
        # 提取特征
        features_list = []
        valid_paths = []
        
        for audio_path in audio_paths:
            try:
                features = self.extract_advanced_features(audio_path)
                features_list.append(features)
                valid_paths.append(audio_path)
            except Exception as e:
                print(f"⚠️ 跳过文件 {audio_path}: {e}")
                continue
        
        if len(features_list) < 2:
            print("❌ 数据不足，无法进行聚类分析")
            return
        
        # 转换为DataFrame
        df = pd.DataFrame(features_list)
        X = df.values
        
        # 标准化
        X_scaled = self.scaler.fit_transform(X)
        
        # 降维
        # Adjust n_components to be at most the minimum of samples and features
        n_components = min(2, min(X_scaled.shape[0], X_scaled.shape[1]))
        self.pca = PCA(n_components=n_components)
        X_pca = self.pca.fit_transform(X_scaled)
        
        # K-means聚类
        kmeans = KMeans(n_clusters=min(5, len(X_pca)), random_state=42)
        clusters = kmeans.fit_predict(X_pca)
        
        # 可视化聚类结果
        plt.figure(figsize=(12, 8))
        
        # 主成分分析可视化
        plt.subplot(2, 2, 1)
        scatter = plt.scatter(X_pca[:, 0], X_pca[:, 1], c=clusters, cmap='viridis')
        plt.title('PCA + K-means Clustering')
        plt.xlabel('Principal Component 1')
        plt.ylabel('Principal Component 2')
        plt.colorbar(scatter)
        
        # 特征重要性
        plt.subplot(2, 2, 2)
        feature_importance = np.abs(self.pca.components_[0])
        top_features_idx = np.argsort(feature_importance)[-10:]
        plt.barh(range(10), feature_importance[top_features_idx])
        plt.yticks(range(10), [df.columns[i] for i in top_features_idx])
        plt.title('Top 10 Feature Importance')
        plt.xlabel('Importance')
        
        # 聚类统计
        plt.subplot(2, 2, 3)
        cluster_counts = np.bincount(clusters)
        plt.bar(range(len(cluster_counts)), cluster_counts)
        plt.title('Cluster Distribution')
        plt.xlabel('Cluster')
        plt.ylabel('Count')
        
        # 特征相关性热图
        plt.subplot(2, 2, 4)
        correlation_matrix = df.corr()
        sns.heatmap(correlation_matrix.iloc[:10, :10], annot=True, cmap='coolwarm', center=0)
        plt.title('Feature Correlation Matrix')
        
        plt.tight_layout()
        plt.show()
        
        return clusters, df
    
    def generate_ai_report(self, audio_paths, labels=None):
        """生成AI分析报告"""
        print("\n" + "="*60)
        print("🤖 高级音频AI分析报告")
        print("="*60)
        
        # 聚类分析
        clusters, features_df = self.analyze_audio_cluster(audio_paths)
        
        # 如果有标签，训练模型
        if labels:
            results, X_test, y_test = self.train_models(audio_paths, labels)
            
            # 显示模型性能
            print(f"\n📊 模型性能对比:")
            for name, result in results.items():
                print(f"   {name:15s}: {result['accuracy']:.3f}")
        
        # 特征统计
        print(f"\n📈 特征统计:")
        print(f"   分析文件数: {len(audio_paths)}")
        print(f"   提取特征数: {len(features_df.columns)}")
        print(f"   聚类数量: {len(np.unique(clusters))}")
        
        # 保存结果
        features_df['cluster'] = clusters
        features_df.to_csv('advanced_audio_analysis.csv', index=False)
        print(f"\n💾 分析结果已保存到 advanced_audio_analysis.csv")
        
        return features_df

def main():
    # 创建高级AI分析器
    ai_analyzer = AdvancedAudioAI()
    
    # 音乐目录路径
    music_dir = 'C:\\Users\\eggy2\\wyc\\数学论文\\音乐'
    
    # 收集所有音频文件
    audio_files = []
    audio_extensions = ['.mp3', '.ogg', '.wav', '.flac']  # 支持的音频扩展名
    
    for root, dirs, files in os.walk(music_dir):
        for file in files:
            if any(file.lower().endswith(ext) for ext in audio_extensions):
                file_path = os.path.join(root, file)
                audio_files.append(file_path)
    
    print(f"🔍 发现 {len(audio_files)} 个音频文件")
    
    try:
        # 生成AI分析报告（无标签）
        results = ai_analyzer.generate_ai_report(audio_files)
        
        print(f"\n✅ 分析完成！")
        
    except Exception as e:
        print(f"❌ 分析过程中出现错误: {str(e)}")
        print("请确保音频文件存在且格式正确")

if __name__ == "__main__":
    main()