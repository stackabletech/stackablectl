CREATE TABLE conditions (
   time         TIMESTAMPTZ     NOT NULL,
   timesecs     NUMERIC(12, 6)  NOT NULL,
   coppm        NUMERIC(12, 6)  NOT NULL,
   humidity     NUMERIC(12, 6)  NOT NULL,
   temperature  NUMERIC(12, 6)  NOT NULL,
   flowrate     NUMERIC(12, 6)  NOT NULL,
   heatervoltage     NUMERIC(12, 6)  NOT NULL,
   r1     NUMERIC(12, 6)  NOT NULL,
   r2     NUMERIC(12, 6)  NOT NULL,
   r3     NUMERIC(12, 6)  NOT NULL,
   r4     NUMERIC(12, 6)  NOT NULL,
   r5     NUMERIC(12, 6)  NOT NULL,
   r6     NUMERIC(12, 6)  NOT NULL,
   r7     NUMERIC(12, 6)  NOT NULL,
   r8     NUMERIC(12, 6)  NOT NULL,
   r9     NUMERIC(12, 6)  NOT NULL,
   r10    NUMERIC(12, 6)  NOT NULL,
   r11    NUMERIC(12, 6)  NOT NULL,
   r12    NUMERIC(12, 6)  NOT NULL,
   r13    NUMERIC(12, 6)  NOT NULL,
   r14    NUMERIC(12, 6)  NOT NULL
);

CREATE UNIQUE INDEX idx_conditions ON conditions(timesecs, time);

SELECT create_hypertable('conditions', 'time');

CREATE TABLE conditions_temp (
   timesecs     NUMERIC(12, 6)  NOT NULL,
   coppm        NUMERIC(12, 6)  NOT NULL,
   humidity     NUMERIC(12, 6)  NOT NULL,
   temperature  NUMERIC(12, 6)  NOT NULL,
   flowrate     NUMERIC(12, 6)  NOT NULL,
   heatervoltage     NUMERIC(12, 6)  NOT NULL,
   r1     NUMERIC(12, 6)  NOT NULL,
   r2     NUMERIC(12, 6)  NOT NULL,
   r3     NUMERIC(12, 6)  NOT NULL,
   r4     NUMERIC(12, 6)  NOT NULL,
   r5     NUMERIC(12, 6)  NOT NULL,
   r6     NUMERIC(12, 6)  NOT NULL,
   r7     NUMERIC(12, 6)  NOT NULL,
   r8     NUMERIC(12, 6)  NOT NULL,
   r9     NUMERIC(12, 6)  NOT NULL,
   r10    NUMERIC(12, 6)  NOT NULL,
   r11    NUMERIC(12, 6)  NOT NULL,
   r12    NUMERIC(12, 6)  NOT NULL,
   r13    NUMERIC(12, 6)  NOT NULL,
   r14    NUMERIC(12, 6)  NOT NULL
);