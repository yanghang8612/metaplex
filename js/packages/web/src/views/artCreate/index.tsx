import React, { useEffect, useState, useCallback } from 'react';
import {
  Steps,
  Row,
  Button,
  Upload,
  Col,
  Input,
  Statistic,
  Slider,
  Progress,
  Spin,
  InputNumber,
} from 'antd';
import { ArtCard } from './../../components/ArtCard';
import { UserSearch, UserValue } from './../../components/UserSearch';
import { Confetti } from './../../components/Confetti';
import './../styles.less';
import { mintNFT } from '../../actions';
import {
  MAX_METADATA_LEN,
  useConnection,
  useWallet,
  IMetadataExtension,
  MetadataCategory,
  useConnectionConfig,
  Creator,
} from '@oyster/common';
import { getAssetCostToStore, LAMPORT_MULTIPLIER } from '../../utils/assets';
import { Connection, PublicKey } from '@solana/web3.js';
import { MintLayout } from '@solana/spl-token';
import { useHistory, useParams } from 'react-router-dom';
import { cleanName } from '../../utils/utils';
import { useSolPrice } from '../../contexts';
import { AmountLabel } from '../../components/AmountLabel';

const { Step } = Steps;
const { Dragger } = Upload;

export const ArtCreateView = () => {
  const connection = useConnection();
  const { env } = useConnectionConfig();
  const { wallet } = useWallet();
  const { step_param }: { step_param: string } = useParams();
  const history = useHistory();

  const [step, setStep] = useState<number>(0);
  const [stepsVisible, setStepsVisible] = useState<boolean>(true);
  const [progress, setProgress] = useState<number>(0);
  const [nft, setNft] =
    useState<{ metadataAccount: PublicKey } | undefined>(undefined);
  const [attributes, setAttributes] = useState<IMetadataExtension>({
    name: '',
    symbol: '',
    description: '',
    externalUrl: '',
    image: '',
    creators: [],
    properties: {
      royalty: 0,
      files: [],
      category: MetadataCategory.Image,
    },
  });

  const gotoStep = useCallback(
    (_step: number) => {
      history.push(`/art/create/${_step.toString()}`);
    },
    [history],
  );

  useEffect(() => {
    if (step_param) setStep(parseInt(step_param));
    else gotoStep(0);
  }, [step_param, gotoStep]);

  // store files
  const mint = async () => {
    const metadata = {
      name: attributes.name,
      symbol: attributes.symbol,
      creators: attributes.creators,
      description: attributes.description,
      image:
        attributes.properties?.files &&
        attributes.properties?.files?.[0] &&
        attributes.properties?.files[0].name,
      external_url: attributes.externalUrl,
      properties: {
        files: (attributes?.properties?.files || []).map(f => f.name),
        category: attributes.properties?.category,
        royalty: attributes.properties?.royalty,
      },
    };
    setStepsVisible(false);
    const inte = setInterval(() => setProgress(prog => prog + 1), 600);
    // Update progress inside mintNFT
    const _nft = await mintNFT(
      connection,
      wallet,
      env,
      attributes?.properties?.files || [],
      metadata,
      attributes.properties?.maxSupply,
    );
    if (_nft) setNft(_nft);
    clearInterval(inte);
  };

  return (
    <>
      <Row style={{ paddingTop: 50 }}>
        {stepsVisible && (
          <Col span={24} md={4}>
            <Steps
              progressDot
              direction="vertical"
              current={step}
              style={{ width: 'fit-content', margin: 'auto' }}
            >
              <Step title="Category" />
              <Step title="Upload" />
              <Step title="Info" />
              <Step title="Royalties" />
              <Step title="Launch" />
            </Steps>
          </Col>
        )}
        <Col span={24} {...(stepsVisible ? { md: 20 } : { md: 24 })}>
          {step === 0 && (
            <CategoryStep
              confirm={(category: MetadataCategory) => {
                setAttributes({
                  ...attributes,
                  properties: {
                    ...attributes.properties,
                    category,
                  },
                });
                gotoStep(1);
              }}
            />
          )}
          {step === 1 && (
            <UploadStep
              attributes={attributes}
              setAttributes={setAttributes}
              confirm={() => gotoStep(2)}
            />
          )}

          {step === 2 && (
            <InfoStep
              attributes={attributes}
              setAttributes={setAttributes}
              confirm={() => gotoStep(3)}
            />
          )}
          {step === 3 && (
            <RoyaltiesStep
              attributes={attributes}
              confirm={() => gotoStep(4)}
              setAttributes={setAttributes}
            />
          )}
          {step === 4 && (
            <LaunchStep
              attributes={attributes}
              confirm={() => gotoStep(5)}
              connection={connection}
            />
          )}
          {step === 5 && (
            <WaitingStep
              mint={mint}
              progress={progress}
              confirm={() => gotoStep(6)}
            />
          )}
          {step === 6 && <Congrats nft={nft} />}
          {0 < step && step < 5 && (
            <div style={{ margin: 'auto', width: 'fit-content' }}>
              <Button onClick={() => gotoStep(step - 1)}>Back</Button>
            </div>
          )}
        </Col>
      </Row>
    </>
  );
};

const CategoryStep = (props: {
  confirm: (category: MetadataCategory) => void;
}) => {
  return (
    <>
      <Row className="call-to-action">
        <h2>Create a new item</h2>
        <p>
          First time creating on Metaplex?{' '}
          <a href="#">Read our creators’ guide.</a>
        </p>
      </Row>
      <Row>
        <Col>
          <Row>
            <Button
              className="type-btn"
              size="large"
              onClick={() => props.confirm(MetadataCategory.Image)}
            >
              <div>
                <div>Image</div>
                <div className="type-btn-description">JPG, PNG, GIF</div>
              </div>
            </Button>
          </Row>
          <Row>
            <Button
              className="type-btn"
              size="large"
              onClick={() => props.confirm(MetadataCategory.Video)}
            >
              <div>
                <div>Video</div>
                <div className="type-btn-description">MP4</div>
              </div>
            </Button>
          </Row>
          <Row>
            <Button
              className="type-btn"
              size="large"
              onClick={() => props.confirm(MetadataCategory.Audio)}
            >
              <div>
                <div>Audio</div>
                <div className="type-btn-description">MP3, WAV, FLAC</div>
              </div>
            </Button>
          </Row>
          <Row>
            <Button
              disabled={true}
              className="type-btn"
              size="large"
              onClick={() => props.confirm(MetadataCategory.Audio)}
            >
              <div>
                <div>AR/3D</div>
                <div className="type-btn-description">Coming Soon</div>
              </div>
            </Button>
          </Row>
        </Col>
      </Row>
    </>
  );
};

const UploadStep = (props: {
  attributes: IMetadataExtension;
  setAttributes: (attr: IMetadataExtension) => void;
  confirm: () => void;
}) => {
  const [mainFile, setMainFile] = useState<any>();
  const [coverFile, setCoverFile] = useState<any>();
  const [image, setImage] = useState<string>('');

  useEffect(() => {
    props.setAttributes({
      ...props.attributes,
      properties: {
        ...props.attributes.properties,
        files: [],
      },
    });
  }, []);

  const uploadMsg = (category: MetadataCategory) => {
    switch (category) {
      case MetadataCategory.Audio:
        return 'Upload your audio creation (MP3, FLAC, WAV)';
      case MetadataCategory.Image:
        return 'Upload your image creation (PNG, JPG, GIF)';
      case MetadataCategory.Video:
        return 'Upload your video creation (MP4)';
      default:
        return 'Please go back and choose a category';
    }
  };

  const acceptableFiles = (category: MetadataCategory) => {
    switch (category) {
      case MetadataCategory.Audio:
        return '.mp3,.flac,.wav';
      case MetadataCategory.Image:
        return '.png,.jpg,.gif';
      case MetadataCategory.Video:
        return '.mp4';
      default:
        return '';
    }
  };

  return (
    <>
      <Row className="call-to-action">
        <h2>Now, let's upload your creation</h2>
        <p style={{ fontSize: '1.2rem' }}>
          Your file will be uploaded to the decentralized web via Arweave.
          Depending on file type, can take up to 1 minute. Arweave is a new type
          of storage that backs data with sustainable and perpetual endowments,
          allowing users and developers to truly store data forever – for the
          very first time.
        </p>
      </Row>
      <Row className="content-action">
        <h3>{uploadMsg(props.attributes.properties?.category)}</h3>
        <Dragger
          accept={acceptableFiles(props.attributes.properties?.category)}
          style={{ padding: 20 }}
          multiple={false}
          customRequest={info => {
            // dont upload files here, handled outside of the control
            info?.onSuccess?.({}, null as any);
          }}
          fileList={mainFile ? [mainFile] : []}
          onChange={async info => {
            const file = info.file.originFileObj;
            if (file) setMainFile(file);
            if (
              props.attributes.properties?.category !== MetadataCategory.Audio
            ) {
              const reader = new FileReader();
              reader.onload = function (event) {
                setImage((event.target?.result as string) || '');
              };
              if (file) reader.readAsDataURL(file);
            }
          }}
        >
          <div className="ant-upload-drag-icon">
            <h3 style={{ fontWeight: 700 }}>Upload your creation</h3>
          </div>
          <p className="ant-upload-text">Drag and drop, or click to browse</p>
        </Dragger>
      </Row>
      {props.attributes.properties?.category === MetadataCategory.Audio && (
        <Row className="content-action">
          <h3>
            Optionally, you can upload a cover image or video (PNG, JPG, GIF,
            MP4)
          </h3>
          <Dragger
            accept=".png,.jpg,.gif,.mp4"
            style={{ padding: 20 }}
            multiple={false}
            customRequest={info => {
              // dont upload files here, handled outside of the control
              info?.onSuccess?.({}, null as any);
            }}
            fileList={coverFile ? [coverFile] : []}
            onChange={async info => {
              const file = info.file.originFileObj;
              if (file) setCoverFile(file);
              if (
                props.attributes.properties?.category === MetadataCategory.Audio
              ) {
                const reader = new FileReader();
                reader.onload = function (event) {
                  setImage((event.target?.result as string) || '');
                };
                if (file) reader.readAsDataURL(file);
              }
            }}
          >
            <div className="ant-upload-drag-icon">
              <h3 style={{ fontWeight: 700 }}>
                Upload your cover image or video (PNG, JPG, GIF, MP4)
              </h3>
            </div>
            <p className="ant-upload-text">Drag and drop, or click to browse</p>
          </Dragger>
        </Row>
      )}
      <Row>
        <Button
          type="primary"
          size="large"
          onClick={() => {
            props.setAttributes({
              ...props.attributes,
              properties: {
                ...props.attributes.properties,
                files: [mainFile, coverFile]
                  .filter(f => f)
                  .map(f => new File([f], cleanName(f.name), { type: f.type })),
              },
              image,
            });
            props.confirm();
          }}
          className="action-btn"
        >
          Continue to Mint
        </Button>
      </Row>
    </>
  );
};

interface Royalty {
  creatorKey: string;
  amount: number;
}

const InfoStep = (props: {
  attributes: IMetadataExtension;
  setAttributes: (attr: IMetadataExtension) => void;
  confirm: () => void;
}) => {
  const [creators, setCreators] = useState<Array<UserValue>>([]);
  const [royalties, setRoyalties] = useState<Array<Royalty>>([]);

  useEffect(() => {
    setRoyalties(
      creators.map(creator => ({
        creatorKey: creator.key,
        amount: Math.trunc(100 / creators.length),
      })),
    );
  }, [creators]);

  return (
    <>
      <Row className="call-to-action">
        <h2>Describe your item</h2>
        <p>
          Provide detailed description of your creative process to engage with
          your audience.
        </p>
      </Row>
      <Row className="content-action" justify="space-around">
        <Col>
          {props.attributes.image && (
            <ArtCard
              image={props.attributes.image}
              category={props.attributes.properties?.category}
              name={props.attributes.name}
              symbol={props.attributes.symbol}
              small={true}
            />
          )}
        </Col>
        <Col className="section" style={{ minWidth: 300 }}>
          <label className="action-field">
            <span className="field-title">Title</span>
            <Input
              autoFocus
              className="input"
              placeholder="Max 50 characters"
              allowClear
              value={props.attributes.name}
              onChange={info =>
                props.setAttributes({
                  ...props.attributes,
                  name: info.target.value,
                })
              }
            />
          </label>
          {/* <label className="action-field">
            <span className="field-title">Symbol</span>
            <Input
              className="input"
              placeholder="Max 10 characters"
              allowClear
              value={props.attributes.symbol}
              onChange={info =>
                props.setAttributes({
                  ...props.attributes,
                  symbol: info.target.value,
                })
              }
            />
          </label> */}
          <label className="action-field">
            <span className="field-title">Creators</span>
            <UserSearch setCreators={setCreators} />
          </label>
          <label className="action-field">
            <span className="field-title">Description</span>
            <Input.TextArea
              className="input textarea"
              placeholder="Max 500 characters"
              value={props.attributes.description}
              onChange={info =>
                props.setAttributes({
                  ...props.attributes,
                  description: info.target.value,
                })
              }
              allowClear
            />
          </label>
        </Col>
      </Row>
      {creators.length > 0 && (
        <Row>
          <label className="action-field" style={{ width: '100%' }}>
            <span className="field-title">Royalties Split</span>
            <RoyaltiesSplitter
              creators={creators}
              royalties={royalties}
              setRoyalties={setRoyalties}
            />
          </label>
        </Row>
      )}
      <Row>
        <Button
          type="primary"
          size="large"
          onClick={() => {
            const creatorStructs: Creator[] = creators.map(
              c =>
                new Creator({
                  address: new PublicKey(c.value),
                  verified: true,
                  share:
                    royalties.find(r => r.creatorKey == c.value)?.amount || 0,
                }),
            );
            props.setAttributes({
              ...props.attributes,
              creators: creatorStructs,
            });

            props.confirm();
          }}
          className="action-btn"
        >
          Continue to royalties
        </Button>
      </Row>
    </>
  );
};

const shuffle = (array: Array<any>) => {
  array.sort(() => Math.random() - 0.5);
};

const RoyaltiesSplitter = (props: {
  creators: Array<UserValue>;
  royalties: Array<Royalty>;
  setRoyalties: Function;
}) => {
  return (
    <Col>
      {props.creators.map((creator, idx) => {
        const royalty = props.royalties.find(
          royalty => royalty.creatorKey === creator.key,
        );
        if (!royalty) return null;

        const amt = royalty.amount;
        const handleSlide = (newAmt: number) => {
          const othersRoyalties = props.royalties.filter(
            _royalty => _royalty.creatorKey !== royalty.creatorKey,
          );
          if (othersRoyalties.length < 1) return;
          shuffle(othersRoyalties);
          const others_n = props.royalties.length - 1;
          const sign = Math.sign(newAmt - amt);
          let remaining = Math.abs(newAmt - amt);
          let count = 0;
          while (remaining > 0 && count < 100) {
            const idx = count % others_n;
            const _royalty = othersRoyalties[idx];
            if (
              (0 < _royalty.amount && _royalty.amount < 100) || // Normal
              (_royalty.amount === 0 && sign < 0) || // Low limit
              (_royalty.amount === 100 && sign > 0) // High limit
            ) {
              _royalty.amount -= sign;
              remaining -= 1;
            }
            count += 1;
          }

          props.setRoyalties(
            props.royalties.map(_royalty => {
              const computed_amount = othersRoyalties.find(
                newRoyalty => newRoyalty.creatorKey === _royalty.creatorKey,
              )?.amount;
              return {
                ..._royalty,
                amount:
                  _royalty.creatorKey === royalty.creatorKey
                    ? newAmt
                    : computed_amount,
              };
            }),
          );
        };
        return (
          <Row key={idx} style={{ margin: '5px auto' }}>
            <Col span={11} className="slider-elem">
              {creator.label}
            </Col>
            <Col span={8} className="slider-elem">
              {amt}%
            </Col>
            <Col span={4}>
              <Slider value={amt} onChange={handleSlide} />
            </Col>
          </Row>
        );
      })}
    </Col>
  );
};

const RoyaltiesStep = (props: {
  attributes: IMetadataExtension;
  setAttributes: (attr: IMetadataExtension) => void;
  confirm: () => void;
}) => {
  const file = props.attributes.image;

  return (
    <>
      <Row className="call-to-action">
        <h2>Set royalties and supply limits for the creation</h2>
        <p>
          A royalty is a payment made by the seller of this item to the creator.
          It is charged after every successful auction.
        </p>
        <p>
          Setting a maximum supply on your NFT is entirely optional, but once
          set, no more than this number of limited editions can ever be printed
          in auctions. Leaving this field blank gives you the latitude to decide
          on a per-auction basis how many prints you wish to make.
        </p>
      </Row>
      <Row className="content-action" justify="space-around">
        <Col>
          {file && (
            <ArtCard
              image={props.attributes.image}
              category={props.attributes.properties?.category}
              name={props.attributes.name}
              symbol={props.attributes.symbol}
              small={true}
            />
          )}
        </Col>
        <Col className="section" style={{ minWidth: 300 }}>
          <label className="action-field">
            <span className="field-title">Royalty Percentage</span>
            <InputNumber
              autoFocus
              min={0}
              max={100}
              placeholder="Between 0 and 100"
              onChange={(val: number) => {
                props.setAttributes({
                  ...props.attributes,
                  properties: {
                    ...props.attributes.properties,
                    royalty: val,
                  },
                });
              }}
              className="royalties-input"
            />
          </label>
          <label className="action-field">
            <span className="field-title">Maximum Supply</span>
            <InputNumber
              placeholder="Quantity"
              onChange={(val: number) => {
                props.setAttributes({
                  ...props.attributes,
                  properties: {
                    ...props.attributes.properties,
                    maxSupply: val,
                  },
                });
              }}
              className="royalties-input"
            />
          </label>
        </Col>
      </Row>
      <Row>
        <Button
          type="primary"
          size="large"
          onClick={props.confirm}
          className="action-btn"
        >
          Continue to review
        </Button>
      </Row>
    </>
  );
};

const LaunchStep = (props: {
  confirm: () => void;
  attributes: IMetadataExtension;
  connection: Connection;
}) => {
  const files = props.attributes.properties?.files || [];
  const metadata = {
    ...(props.attributes as any),
    files: files.map(f => f?.name),
  };
  const [cost, setCost] = useState(0);
  const [USDcost, setUSDcost] = useState(0);
  const solPrice = useSolPrice();
  useEffect(() => {
    const rentCall = Promise.all([
      props.connection.getMinimumBalanceForRentExemption(MintLayout.span),
      props.connection.getMinimumBalanceForRentExemption(MAX_METADATA_LEN),
    ]);

    getAssetCostToStore([
      ...files,
      new File([JSON.stringify(metadata)], 'metadata.json'),
    ]).then(async lamports => {
      const sol = lamports / LAMPORT_MULTIPLIER;

      // TODO: cache this and batch in one call
      const [mintRent, metadataRent] = await rentCall;

      // const uriStr = 'x';
      // let uriBuilder = '';
      // for (let i = 0; i < MAX_URI_LENGTH; i++) {
      //   uriBuilder += uriStr;
      // }

      const additionalSol = (metadataRent + mintRent) / LAMPORT_MULTIPLIER;

      // TODO: add fees based on number of transactions and signers
      setCost(sol + additionalSol);
    });
  }, [files, setCost]);

  useEffect(() => {
    cost && setUSDcost(solPrice * cost);
  }, [cost, solPrice]);

  return (
    <>
      <Row className="call-to-action">
        <h2>Launch your creation</h2>
        <p>
          Provide detailed description of your creative process to engage with
          your audience.
        </p>
      </Row>
      <Row className="content-action" justify="space-around">
        <Col>
          {props.attributes.image && (
            <ArtCard
              image={props.attributes.image}
              category={props.attributes.properties?.category}
              name={props.attributes.name}
              symbol={props.attributes.symbol}
              small={true}
            />
          )}
        </Col>
        <Col className="section" style={{ minWidth: 300 }}>
          <Statistic
            className="create-statistic"
            title="Royalty Percentage"
            value={props.attributes.properties?.royalty}
            suffix="%"
          />
          {cost ? (
            <AmountLabel title="Cost to Create" amount={cost} />
          ) : (
            <Spin />
          )}
        </Col>
      </Row>
      <Row>
        <Button
          type="primary"
          size="large"
          onClick={props.confirm}
          className="action-btn"
        >
          Pay with SOL
        </Button>
        <Button
          disabled={true}
          size="large"
          onClick={props.confirm}
          className="action-btn"
        >
          Pay with Credit Card
        </Button>
      </Row>
    </>
  );
};

const WaitingStep = (props: {
  mint: Function;
  progress: number;
  confirm: Function;
}) => {
  useEffect(() => {
    const func = async () => {
      await props.mint();
      props.confirm();
    };
    func();
  }, []);

  return (
    <div
      style={{
        marginTop: 70,
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
      }}
    >
      <Progress type="circle" percent={props.progress} />
      <div className="waiting-title">
        Your creation is being uploaded to the decentralized web...
      </div>
      <div className="waiting-subtitle">This can take up to 1 minute.</div>
    </div>
  );
};

const Congrats = (props: {
  nft?: {
    metadataAccount: PublicKey;
  };
}) => {
  const history = useHistory();

  const newTweetURL = () => {
    const params = {
      text: "I've created a new NFT artwork on Metaplex, check it out!",
      url: `${
        window.location.origin
      }/#/art/${props.nft?.metadataAccount.toString()}`,
      hashtags: 'NFT,Crypto,Metaplex',
      // via: "Metaplex",
      related: 'Metaplex,Solana',
    };
    const queryParams = new URLSearchParams(params).toString();
    return `https://twitter.com/intent/tweet?${queryParams}`;
  };

  return (
    <>
      <div
        style={{
          marginTop: 70,
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
        }}
      >
        <div className="waiting-title">
          Congratulations, you created an NFT!
        </div>
        <div className="congrats-button-container">
          <Button
            className="metaplex-button"
            onClick={_ => window.open(newTweetURL(), '_blank')}
          >
            <span>Share it on Twitter</span>
            <span>&gt;</span>
          </Button>
          <Button
            className="metaplex-button"
            onClick={_ =>
              history.push(`/art/${props.nft?.metadataAccount.toString()}`)
            }
          >
            <span>See it in your collection</span>
            <span>&gt;</span>
          </Button>
          <Button
            className="metaplex-button"
            onClick={_ => history.push('/auction/create')}
          >
            <span>Sell it via auction</span>
            <span>&gt;</span>
          </Button>
        </div>
      </div>
      <Confetti />
    </>
  );
};
