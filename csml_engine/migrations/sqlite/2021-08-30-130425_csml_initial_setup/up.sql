
CREATE TABLE cmsl_bot_versions (
  id BINARY(128) PRIMARY KEY NOT NULL,
  bot_id VARCHAR NOT NULL,

  bot TEXT NOT NULL,
  engine_version VARCHAR NOT NULL,

  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE csml_conversations (
  id BINARY(128) PRIMARY KEY NOT NULL,
  bot_id VARCHAR NOT NULL,
  channel_id VARCHAR NOT NULL,
  user_id VARCHAR NOT NULL,

  flow_id VARCHAR NOT NULL,
  step_id VARCHAR NOT NULL,
  status VARCHAR NOT NULL,

  last_interaction_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  expires_at TIMESTAMP DEFAULT NULL
);


CREATE TABLE csml_messages (
  id BINARY(128) PRIMARY KEY NOT NULL,
  conversation_id BINARY(128) NOT NULL REFERENCES csml_conversations (id) ON DELETE CASCADE,

  flow_id VARCHAR NOT NULL,
  step_id VARCHAR NOT NULL,
  direction VARCHAR NOT NULL,
  payload VARCHAR NOT NULL,
  content_type VARCHAR NOT NULL,

  message_order INTEGER NOT NULL,
  interaction_order INTEGER NOT NULL,

  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  expires_at TIMESTAMP DEFAULT NULL
);

CREATE TABLE csml_memories (
  id BINARY(128) PRIMARY KEY NOT NULL,
  bot_id VARCHAR NOT NULL,
  channel_id VARCHAR NOT NULL,
  user_id VARCHAR NOT NULL,

  key VARCHAR NOT NULL,
  value VARCHAR NOT NULL,

  expires_at TIMESTAMP DEFAULT NULL,

  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX memory_client_key ON csml_memories (bot_id, channel_id, user_id, key);


CREATE TABLE csml_states (
  id BINARY(128) PRIMARY KEY NOT NULL,
  bot_id VARCHAR NOT NULL,
  channel_id VARCHAR NOT NULL,
  user_id VARCHAR NOT NULL,

  type VARCHAR NOT NULL,
  key VARCHAR NOT NULL,
  value VARCHAR NOT NULL,

  expires_at TIMESTAMP DEFAULT NULL,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
