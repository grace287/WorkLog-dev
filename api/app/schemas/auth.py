from pydantic import BaseModel


class TokenRequest(BaseModel):
    github_pat: str


class TokenResponse(BaseModel):
    access_token: str
    token_type: str = "bearer"
