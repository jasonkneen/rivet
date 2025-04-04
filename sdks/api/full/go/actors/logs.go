// This file was auto-generated by Fern from our API Definition.

package actors

import (
	json "encoding/json"
	fmt "fmt"
	sdk "sdk"
	core "sdk/core"
)

type GetActorLogsRequestQuery struct {
	Project     *string   `json:"-"`
	Environment *string   `json:"-"`
	Stream      LogStream `json:"-"`
	// A query parameter denoting the requests watch index.
	WatchIndex *string `json:"-"`
}

type GetActorLogsResponse struct {
	// Sorted old to new.
	Lines []string `json:"lines,omitempty"`
	// Sorted old to new.
	Timestamps []sdk.Timestamp    `json:"timestamps,omitempty"`
	Watch      *sdk.WatchResponse `json:"watch,omitempty"`

	_rawJSON json.RawMessage
}

func (g *GetActorLogsResponse) UnmarshalJSON(data []byte) error {
	type unmarshaler GetActorLogsResponse
	var value unmarshaler
	if err := json.Unmarshal(data, &value); err != nil {
		return err
	}
	*g = GetActorLogsResponse(value)
	g._rawJSON = json.RawMessage(data)
	return nil
}

func (g *GetActorLogsResponse) String() string {
	if len(g._rawJSON) > 0 {
		if value, err := core.StringifyJSON(g._rawJSON); err == nil {
			return value
		}
	}
	if value, err := core.StringifyJSON(g); err == nil {
		return value
	}
	return fmt.Sprintf("%#v", g)
}

type LogStream string

const (
	LogStreamStdOut LogStream = "std_out"
	LogStreamStdErr LogStream = "std_err"
)

func NewLogStreamFromString(s string) (LogStream, error) {
	switch s {
	case "std_out":
		return LogStreamStdOut, nil
	case "std_err":
		return LogStreamStdErr, nil
	}
	var t LogStream
	return "", fmt.Errorf("%s is not a valid %T", s, t)
}

func (l LogStream) Ptr() *LogStream {
	return &l
}
