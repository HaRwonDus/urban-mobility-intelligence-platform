from fastapi import FastAPI
from pydantic import BaseModel, Field

app = FastAPI(title="Urban Mobility AI Service", version="0.1.0")


class AreaMetrics(BaseModel):
    area: str
    object_density: float = Field(ge=0)
    avg_time_to_stop: float = Field(ge=0)
    avg_time_to_hub: float = Field(ge=0)
    route_overlap_index: float = Field(ge=0)


class Recommendation(BaseModel):
    area: str
    problem: str
    recommendation: str
    confidence: float = Field(ge=0, le=1)
    model_name: str = "rules-v1"


@app.get("/health")
def health() -> dict[str, str]:
    return {"status": "ok", "service": "urban-mobility-ai"}


@app.post("/recommend", response_model=list[Recommendation])
def recommend(metrics: list[AreaMetrics]) -> list[Recommendation]:
    recommendations: list[Recommendation] = []

    for item in metrics:
        if item.avg_time_to_stop >= 15 and item.object_density >= 0.65:
            recommendations.append(
                Recommendation(
                    area=item.area,
                    problem="High activity density with weak access to trunk transport",
                    recommendation="Evaluate an express route or dedicated feeder line for this area.",
                    confidence=0.82,
                )
            )

        if item.avg_time_to_hub >= 20:
            recommendations.append(
                Recommendation(
                    area=item.area,
                    problem="Long average time to transfer hub",
                    recommendation="Consider a transfer hub near the strongest stop cluster.",
                    confidence=0.74,
                )
            )

        if item.route_overlap_index >= 0.75:
            recommendations.append(
                Recommendation(
                    area=item.area,
                    problem="Likely route duplication",
                    recommendation="Audit overlapping routes and move capacity toward underserved corridors.",
                    confidence=0.69,
                )
            )

    return recommendations

