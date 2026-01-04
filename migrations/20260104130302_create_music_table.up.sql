-- Add up migration script here
-- 音乐信息表
CREATE TABLE musics (
    -- 主键，自增ID
    id INTEGER PRIMARY KEY AUTOINCREMENT,

    -- B站视频ID，唯一标识，用于防重复
    bvid VARCHAR(255) NOT NULL UNIQUE,

    -- 歌曲名称，支持全文搜索
    song_name TEXT NOT NULL,

    -- 视频CID，用于音频流获取
    cid VARCHAR(255) NOT NULL,

    -- 作者/UP主名称
    author TEXT NOT NULL,

    -- 是否喜欢标记，1=喜欢，0=未标记
    is_liked BOOLEAN NOT NULL DEFAULT 0,

    -- 软删除标记，1=已删除，0=未删除
    is_deleted BOOLEAN NOT NULL DEFAULT 0,

    -- 记录创建时间，自动填充
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,

    -- 最后更新时间
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 创建索引优化查询性能
-- 1. 复合索引：常用查询条件组合
CREATE INDEX idx_musics_liked_deleted ON musics(is_liked, is_deleted, created_at DESC);

-- 2. 复合索引：歌曲搜索和列表查询
CREATE INDEX idx_musics_search ON musics(song_name, author, is_deleted);

-- 3. 单字段索引：常用于排序和范围查询
CREATE INDEX idx_musics_created_at ON musics(created_at DESC);

-- 4. 单字段索引：作者查询
CREATE INDEX idx_musics_author ON musics(author, is_deleted);

-- 5. 软删除查询优化
CREATE INDEX idx_musics_deleted ON musics(is_deleted, created_at DESC);

-- 6. 唯一性索引（bvid已有UNIQUE约束，此索引可省略，SQLite会自动创建）
-- CREATE UNIQUE INDEX idx_musics_bvid ON musics(bvid);

-- 如果需要频繁进行歌曲名称的模糊搜索，考虑以下优化：
-- 注意：SQLite的LIKE查询在索引使用上有一定限制
-- CREATE INDEX idx_musics_song_name_prefix ON musics(song_name COLLATE NOCASE);

-- 创建更新时间的触发器
CREATE TRIGGER update_musics_timestamp
AFTER UPDATE ON musics
BEGIN
    UPDATE musics
    SET updated_at = CURRENT_TIMESTAMP
    WHERE id = NEW.id;
END;

-- 添加表注释（SQLite不支持COMMENT ON TABLE，可通过以下方式记录）
-- 表用途：存储B站音乐视频信息，支持喜欢标记和软删除功能
-- 设计原则：
-- 1. bvid作为业务唯一标识，避免重复添加
-- 2. 使用软删除而非物理删除，保留数据可恢复性
-- 3. 索引设计覆盖常用查询场景
-- 4. 时间戳记录创建和更新，便于数据追踪
