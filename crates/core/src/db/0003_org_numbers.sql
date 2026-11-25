-- Handelsregister-Nr und Kostenstellen-Nr beim Betrieb (vom Sekretariat gepflegt).
ALTER TABLE organisations ADD COLUMN commercial_register_no TEXT;
ALTER TABLE organisations ADD COLUMN cost_center_no TEXT;
