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
			'content', json(messages.content), 
			'created', messages.created_at)
		  WHEN messages.sender IS NOT NULL THEN 
		    json_object(
			'sender', coalesce(
				(SELECT display_name FROM contacts WHERE session_id = messages.sender), 
				messages.sender
			), 
			'content', json(messages.content), 
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
	(convo.type = 'one_to_one' AND (messages.sender = convo.id AND messages.receiver = identities.session_id) OR (messages.receiver = convo.id AND messages.sender = identities.session_id)) 
			OR (messages.sender = convo.id OR messages.receiver = convo.id)
)
LEFT JOIN messages mc ON (
	(mc.created_at > convo.last_read) AND (
		(convo.type = 'one_to_one' AND mc.sender = convo.id AND mc.receiver = identities.session_id) OR 
		(convo.type = 'group' AND mc.receiver = convo.id AND mc.sender != identities.session_id) OR 
		(convo.type = 'community' AND mc.receiver = convo.id AND mc.sender != community_identity.value)
	)
)
WHERE convo.id IS NOT NULL
GROUP BY convo.id
ORDER BY coalesce(contacts.priority, 0) DESC, last_message_created DESC, name ASC
