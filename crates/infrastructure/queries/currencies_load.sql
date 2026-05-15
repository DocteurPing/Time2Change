-- Loads all persisted currencies sorted by code.
SELECT currency, name
FROM   currencies
ORDER  BY currency ASC
