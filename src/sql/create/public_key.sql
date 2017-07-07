INSERT INTO public_key (owner, name, fingerprint, content) VALUES ($1, $2, $3, $4) RETURNING id;
