CREATE TABLE messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source TEXT NOT NULL COLLATE NOCASE,
    hash TEXT NULL DEFAULT NULL,
    content TEXT NOT NULL,
    sender TEXT NOT NULL COLLATE NOCASE,
    receiver TEXT NOT NULL COLLATE NOCASE,
    created_at TIMESTAMP NOT NULL,
    expiration_at TIMESTAMP NOT NULL,
    quoting_timestamp INTEGER DEFAULT NULL,
    job_state TEXT NOT NULL DEFAULT 'none',
    last_job_attempt TIMESTAMP DEFAULT NULL,
    last_job_error TEXT DEFAULT NULL,
    CONSTRAINT valid_source CHECK (
        source LIKE '05%'
        OR source LIKE '03%'
        OR source LIKE 'http://%'
        OR source LIKE 'https://%'
    ),
    CONSTRAINT one_to_one_id_constraint CHECK (
        source NOT LIKE '05%'
        OR (
            sender LIKE '05%'
            AND receiver LIKE '05%'
        )
    ),
    CONSTRAINT group_id_constraint CHECK (
        source NOT LIKE '03%'
        OR (
            sender LIKE '05%'
            AND receiver = source
        )
    ),
    CONSTRAINT community_id_constraint CHECK (
        (
            source NOT LIKE 'http://%'
            AND source NOT LIKE 'https://%'
        )
        OR (
            sender LIKE '15%'
            AND (
                receiver = source
                OR receiver LIKE '15%'
            )
        )
    ),
    CONSTRAINT valid_message_content CHECK (nullif(content, '') IS NOT NULL),
    CONSTRAINT job_state CHECK (
        job_state IN (
            'none',
            'pending_remove',
            'pending_send',
            'failed_remove',
            'failed_send'
        )
    ),
    CONSTRAINT job_state_pending_send CHECK (
        job_state != 'pending_send'
        OR (
            hash IS NULL
            AND last_job_attempt IS NULL
            AND last_job_error IS NULL
        )
    ),
    CONSTRAINT job_state_pending_remove CHECK (
        job_state != 'pending_remove'
        OR (
            last_job_attempt IS NULL
            AND last_job_error IS NULL
        )
    ),
    CONSTRAINT job_state_failed CHECK (
        job_state NOT IN ('failed_remove', 'failed_send')
        OR (
            last_job_attempt IS NOT NULL
            AND last_job_error IS NOT NULL
        )
    ),
    CONSTRAINT synced_message_state CHECK (
        job_state != 'none'
        OR hash IS NOT NULL
    )
);

CREATE UNIQUE INDEX messages_source_hash ON messages (source, hash);

CREATE UNIQUE INDEX messages_source_created ON messages (source, created_at);

CREATE INDEX messages_sender ON messages (sender);

CREATE INDEX messages_receiver ON messages (receiver);

CREATE INDEX messages_quoting_timestamp ON messages (quoting_timestamp);

CREATE INDEX messages_source ON messages (source);

CREATE INDEX message_expr_is_data_message ON messages (
    json_type (content ->> '$.dataMessage') = 'object'
);

CREATE TABLE app_settings (
    name TEXT NOT NULL,
    id TEXT NOT NULL DEFAULT '',
    value TEXT NOT NULL,
    PRIMARY KEY (name, id)
);

CREATE TABLE message_attachments (
    message_id INTEGER NOT NULL REFERENCES messages (id) ON DELETE CASCADE,
    id TEXT NOT NULL PRIMARY KEY,
    url TEXT NOT NULL,
    content_type TEXT NOT NULL,
    content BLOB NOT NULL
);

CREATE INDEX message_attachments_message_local_id ON message_attachments (message_id);

CREATE TABLE message_reactions (
    message_id INTEGER NOT NULL REFERENCES messages (id) ON DELETE CASCADE,
    sender TEXT NOT NULL CHECK (
        sender LIKE '05%'
        OR sender LIKE '15%'
    ) COLLATE NOCASE,
    emoji TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL,
    PRIMARY KEY (message_id, sender, emoji)
);

CREATE INDEX message_reactions_message_local_id ON message_reactions (message_id);

CREATE TABLE configs (
    config_type TEXT NOT NULL,
    id TEXT NOT NULL DEFAULT '',
    value TEXT NOT NULL,
    dump BLOB,
    CONSTRAINT value_is_json CHECK (json_type (value) IN ('object', 'array')),
    PRIMARY KEY (config_type, id)
);

CREATE INDEX configs_config_type ON configs (config_type);
CREATE INDEX config_value_id ON configs(value->>'$.id');
CREATE INDEX config_value_session_id ON configs(value->>'$.session_id');
CREATE INDEX config_value_url ON configs(value->>'$.url');
CREATE INDEX config_value_type ON configs(value->>'$.type');

CREATE TABLE message_retrieve_state (
    source TEXT NOT NULL COLLATE NOCASE,
    namespace INTEGER NOT NULL,
    last_message_hash TEXT NOT NULL,
    CONSTRAINT valid_source CHECK (
        source LIKE '05%'
        OR source LIKE '03%'
        OR source LIKE 'http://%'
        OR source LIKE 'https://%'
    ),
    PRIMARY KEY (source, namespace)
);

-- User profile config view
CREATE VIEW config_user_profile AS
SELECT
    value ->> '$.blinded_msgreqs' AS blinded_msgreqs,
    value ->> '$.name' AS name,
    value ->> '$.nts_expiry' AS nts_expiry,
    value ->> '$.nts_priority' AS nts_priority,
    value ->> '$.profile_pic' AS profile_pic
FROM
    configs
WHERE
    config_type = 'UserProfileConfig'
    AND configs.id = '';

-- Contact view
CREATE VIEW config_contacts AS
SELECT
    ea.value ->> '$.name' AS name,
    ea.value ->> '$.nickname' AS nickname,
    ea.value ->> '$.session_id' AS session_id,
    ea.value ->> '$.approved' AS approved,
    ea.value ->> '$.approved_me' AS approved_me,
    ea.value ->> '$.blocked' AS blocked,
    ea.value ->> '$.profile_picture' AS profile_picture,
    ea.value ->> '$.priority' AS priority
FROM
    configs,
    json_each (configs.value) ea
WHERE
    config_type = 'ContactsConfig'
    AND configs.id = '';

-- ConvoInfo view
CREATE VIEW config_convo_info AS
SELECT
    jt.value ->> '$.last_read' AS last_read,
    jt.value ->> '$.session_id' AS session_id,
    jt.value ->> '$.id' AS id,
    jt.value ->> '$.type' AS type,
    jt.value ->> '$.unread' AS unread,
    jt.value ->> '$.url' AS community_url
FROM
    configs c,
    json_each (c.value) jt
WHERE
    c.config_type = 'ConvoInfoVolatileConfig'
    AND c.id = '';

-- Group config view
CREATE VIEW config_user_groups AS
SELECT
    jt.value ->> '$.id' AS id,
    jt.value ->> '$.type' AS type,
    jt.value ->> '$.url' AS community_url,
    jt.value ->> '$.pub_key' AS community_pub_key,
    jt.value ->> '$.auth_data' AS auth_data,
    jt.value ->> '$.secret_key' AS secret_key,
    jt.value ->> '$.invited' AS invited,
    jt.value ->> '$.joined_at' AS joined_at,
    jt.value ->> '$.mute_until' AS mute_until,
    jt.value ->> '$.name' AS name,
    jt.value ->> '$.notifications' AS notifications,
    jt.value ->> '$.priority' AS priority
FROM
    configs c,
    json_each (c.value) jt
WHERE
    c.config_type = 'UserGroupsConfig'
    AND c.id = '';

-- Group info view
CREATE VIEW config_group_info AS
SELECT
    id AS group_id,
    value ->> '$.name' AS name,
    value ->> '$.description' AS description,
    value ->> '$.delete_attach_before' AS delete_attach_before,
    value ->> '$.delete_before' AS delete_before,
    value ->> '$.expiry_timer' AS expiry_timer,
    value ->> '$.created' AS created,
    value ->> '$.profile_pic' AS profile_pic
FROM
    configs c
WHERE
    c.config_type = 'GroupInfoConfig';

-- Group members view
CREATE VIEW config_group_members AS
SELECT
    c.id AS group_id,
    je.value ->> '$.session_id' AS session_id,
    je.value ->> '$.admin' AS admin,
    je.value ->> '$.invite_status' AS invite_status,
    je.value ->> '$.name' AS name,
    je.value ->> '$.profile_picture' AS profile_picture,
    je.value ->> '$.promotion_status' AS promotion_status,
    je.value ->> '$.removed_status' AS removed_status,
    je.value ->> '$.supplement' AS supplement
FROM
    configs c,
    json_each (c.value) as je
WHERE
    c.config_type = 'GroupMemberConfig';


-- Conversation view
CREATE VIEW conversations AS
WITH
convo AS (
  SELECT
	coalesce(nullif(session_id, ''), nullif(id, ''), nullif(community_url, '')) AS id,
	type,
	unread,
	last_read
  FROM config_convo_info
  WHERE type != 'community'
  UNION
  SELECT
	community_url AS id,
	type,
	0 AS unread,
	0 AS last_read
  FROM config_user_groups
  WHERE type = 'community'
),
identities AS (
	SELECT value->>'$.session_id' AS session_id
	FROM app_settings
	WHERE name = 'identity' AND id = ''
	LIMIT 1
),
contacts AS (
	SELECT
	  coalesce(nullif(nickname, ''), nullif(name, '')) AS display_name,
	  priority, session_id, approved, approved_me, blocked,
	  (
		CASE json_type(profile_picture)
		 WHEN 'object' THEN json_patch(profile_picture, json_object('fallback_text', coalesce(nullif(nickname, ''), nullif(name, ''))))
		 ELSE json_object('fallback_text', coalesce(nullif(nickname, ''), nullif(name, '')))
		END
	  ) AS avatar
	FROM config_contacts
),
group_members AS (
	SELECT
		gm.group_id, gm.session_id, gm.admin, gm.invite_status, gm.promotion_status, gm.removed_status, gm.supplement,
		(identities.session_id = gm.session_id) AS is_me,
		json_patch(
			CASE json_type(gm.profile_picture)
			 WHEN 'object' THEN gm.profile_picture
			 ELSE coalesce(contacts.avatar, '{}')
			END,
			json_object(
				'fallback_text', coalesce(contacts.display_name, gm.name),
				'is_admin', gm.admin,
				'is_me', identities.session_id = gm.session_id
			)
		) AS avatar,
		coalesce(contacts.display_name, name) AS display_name
	FROM config_group_members gm
	LEFT JOIN contacts ON contacts.session_id = gm.session_id
	LEFT JOIN identities
),
ginfo AS (
	SELECT
		group_id, name, description, delete_attach_before, delete_before, expiry_timer, created,
		CASE json_type(g.profile_pic)
		 WHEN 'object' THEN g.profile_pic
		 ELSE (
			SELECT json_group_array(json(avatar))
			FROM (SELECT * FROM group_members ORDER BY is_me DESC, admin DESC, display_name ASC, session_id ASC LIMIT 10)
		 )
		END AS avatar
	FROM config_group_info g
)
SELECT
	convo.id,
	coalesce(
		nullif(contacts.display_name, ''),
		nullif(cinfo.name, ''),
		nullif(ginfo.name, ''),
		nullif(ugroup.name, ''),
		''
	) AS name,
	MAX(messages.created_at) AS last_message_created,
	(
		CASE
		  WHEN (convo.type = 'community' AND messages.sender = community_identity.value) OR (messages.sender = identities.session_id) THEN
			json_object(
			'from_me', true,
			'content', messages.content,
			'created', messages.created_at)
		  WHEN messages.sender IS NOT NULL THEN
		    json_object(
			'sender', coalesce(
				(SELECT display_name FROM contacts WHERE session_id = messages.sender),
				messages.sender
			),
			'content', messages.content,
			'created', messages.created_at)
		  ELSE NULL
		END
	) AS last_message,
	coalesce(contacts.avatar, ginfo.avatar) AS avatar,
	(
		CASE convo.type
		 WHEN 'one_to_one' THEN coalesce(contacts.approved, 0)
		 WHEN 'group' THEN (gm.invite_status = 3 OR gm.invite_status = 0)
		 ELSE 1
		END
	) AS approved,
	COUNT(mc.hash) AS unread_count
FROM convo
LEFT JOIN contacts ON convo.type = 'one_to_one' AND contacts.session_id = convo.id
LEFT JOIN config_user_groups ugroup ON convo.type = 'group' AND ugroup.id = convo.id AND ugroup.type = 'group'
LEFT JOIN ginfo ON convo.type = 'group' AND ginfo.group_id = convo.id
LEFT JOIN config_user_groups cinfo ON convo.type = 'community' AND cinfo.community_url = convo.id AND cinfo.type = 'community'
LEFT JOIN identities ON convo.type != 'community'
LEFT JOIN app_settings community_identity ON convo.type = 'community' AND community_identity.name = 'blinded_id' AND community_identity.id = cinfo.community_url
LEFT JOIN group_members gm ON convo.type = 'group' AND gm.group_id = convo.id AND gm.session_id = identities.session_id
LEFT JOIN messages ON (
	(nullif(messages.content, '') IS NOT NULL) AND
	(convo.type = 'one_to_one' AND
		(messages.sender = convo.id AND messages.receiver = identities.session_id) OR (messages.receiver = convo.id AND messages.sender = identities.session_id)
	) OR
	(messages.receiver = convo.id)
)
LEFT JOIN messages mc ON (
	(mc.created_at > convo.last_read) AND
	(nullif(mc.content, '') IS NOT NULL) AND
	(
		(convo.type = 'one_to_one' AND mc.sender = convo.id AND mc.receiver = identities.session_id) OR
		(convo.type = 'group' AND mc.receiver = convo.id AND mc.sender != identities.session_id) OR
		(convo.type = 'community' AND mc.receiver = convo.id AND mc.sender != community_identity.value)
	)
)
WHERE convo.id IS NOT NULL
GROUP BY convo.id
ORDER BY coalesce(contacts.priority, 0) DESC, last_message_created DESC, name ASC
