-- miax_devデータベースが存在しない場合のみ作成
DO $$
BEGIN
    IF NOT EXISTS (SELECT FROM pg_database WHERE datname = 'miax_dev') THEN
        CREATE DATABASE miax_dev;
    END IF;
END
$$;
